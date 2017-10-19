// Rocket imports
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use rocket_contrib::{Json, Value};

use rocket::State;
use std::sync::Mutex;

use rocket::response::status;
use rocket::http::Status;

// Elastic imports

extern crate elastic;
#[macro_use]
extern crate elastic_derive;

use elastic::prelude::*;
use elastic::client::SyncClient;

use std::str::FromStr;

// Other imports
extern crate uuid;
use uuid::{Uuid, Simple};


// We need to define our DB connection for state storage
type SharedClient = Mutex<SyncClient>;


// Client code

type ElasticId = String;

#[derive(Debug, Serialize, Deserialize)]
enum ItemCategories {
    Phone,
    Tablet
}
impl KeywordFieldType<DefaultKeywordMapping> for ItemCategories {}
impl FromStr for ItemCategories {
    type Err = String;

    fn from_str(s: &str) -> Result<ItemCategories, String> {
        match s {
            "phone" => Ok(ItemCategories::Phone),
            "tablet" => Ok(ItemCategories::Tablet),
            _ => Err(format!("Invalid item category; {}", s)),
        }
    }
}

fn new_elastic_id<'a> () -> Id<'a> {
    id(format!("{}", Uuid::new_v4().simple()))
}

#[derive(Debug, Serialize, Deserialize, ElasticType)]
struct ItemElastic {
    category: ItemCategories,
    description: String,
    dimension_x: i32,
    dimension_y: i32,
    dimension_z: i32,
    make: String,
    model: String,
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ItemClient {
    category: String,
    description: String,
    dimensions: [i32; 3],
    make: String,
    model: String,
    version: String,
}

fn item_client_to_elastic(item_client: ItemClient) -> Result<ItemElastic, String> {
    let item = item_client;

    // Store dimensions in descending order
    let mut sorted_dimensions = item.dimensions.clone();
    sorted_dimensions.sort();

    let category_enum = item.category.parse::<ItemCategories>()?;
    let name = format!("{} {} ({})", &item.model, &item.version, &item.make);

    let item_elastic = ItemElastic {
        category: category_enum,
        description: item.description,
        dimension_x: sorted_dimensions[2],
        dimension_y: sorted_dimensions[1],
        dimension_z: sorted_dimensions[0],
        make: item.make,
        model: item.model,
        name: name,
        version: item.version,
    };
    Ok(item_elastic)
}

#[post("/", format = "application/json", data = "<item_client>")]
fn item_create(item_client: Json<ItemClient>, shared_client: State<SharedClient>) -> Result<Json<Value>, status::Custom<String>> {

    let item_elastic = match item_client_to_elastic(item_client.into_inner()) {
        Ok(i)  => i,
        Err(err) => return Err(status::Custom(Status::BadRequest, err)),
    };
    let client = match shared_client.lock() {
        Ok(c) => c,
        Err(err) => return Err(status::Custom(Status::InternalServerError, format!("{}", err))),
    };

    println!("{:?}", &item_elastic);

    let response = match client.document_index(index("items"), id(new_elastic_id()), item_elastic).send() {
        Ok(r) => r,
        Err(err) => return Err(status::Custom(Status::BadGateway, format!("{}", err))),
    };

    println!("{:?}", &response);
    Ok(Json(json!({"id": response.id()})))
}

#[get("/<item_id>")]
fn item_get(item_id: ElasticId, shared_client: State<SharedClient>) -> Result<Json<Value>, status::Custom<String>> {
    let client = match shared_client.lock() {
        Ok(c) => c,
        Err(err) => return Err(status::Custom(Status::InternalServerError, format!("{}", err))),
    };

    let response = match client.document_get::<ItemElastic>(index("items"), id(item_id)).send() {
        Ok(r) => r,
        Err(err) => return Err(status::Custom(Status::BadGateway, format!("{}", err))),
    };

    let doc = match response.into_document() {
        Some(d) => d,
        None => return Err(status::Custom(Status::NotFound, String::from("Item does not exist"))),
    };
    Ok(Json(json!(doc)))
}



#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

fn rocket() -> rocket::Rocket {

    let builder = SyncClientBuilder::new()
        .base_url("http://localhost:9200")
        .params(|p| p
            .url_param("pretty", true));

    let client = builder.build().expect("Could not build elastic client");

    rocket::ignite()
        .mount("/items", routes![item_create, item_get])
        .catch(errors![not_found])
        .manage(Mutex::new(client))
}

fn main() {
    rocket().launch();
}