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

#[derive(Debug, Serialize, Deserialize, ElasticType)]
struct ItemDimensions {
    x: i32,
    y: i32,
    z: i32
}

#[derive(Debug, Serialize, Deserialize, ElasticType)]
struct ItemElasticComputed {
    unique_name: String
}

#[derive(Debug, Serialize, Deserialize, ElasticType)]
struct ItemElasticData {
    category: ItemCategories,
    make: String,
    model: String,
    version: String,
    description: String,
    dimensions: ItemDimensions
}

#[derive(Debug, Serialize, Deserialize, ElasticType)]
struct ItemElastic {
    computed: ItemElasticComputed,
    data: ItemElasticData
}

#[derive(Debug, Serialize, Deserialize)]
struct ItemClient {
    category: String,
    make: String,
    model: String,
    version: String,
    description: String,
    dimensions: [i32; 3]
}

fn item_client_to_elastic(item_client: ItemClient) -> Result<ItemElastic, String> {
    let item = item_client;

    // Store dimensions in descending order
    let mut sorted_dimensions = item.dimensions.clone();
    sorted_dimensions.sort();

    let category_enum = item.category.parse::<ItemCategories>()?;

    let item_elastic = ItemElastic {
        computed: ItemElasticComputed {
            unique_name: format!("{} {} ({})", item.model, item.version, item.make)
        },
        data: ItemElasticData {
            category: category_enum,
            make: item.make,
            model: item.model,
            version: item.version,
            description: item.description,
            dimensions: ItemDimensions {
                x: sorted_dimensions[2],
                y: sorted_dimensions[1],
                z: sorted_dimensions[0]
            }
        }
    };
    Ok(item_elastic)
}

#[post("/", format = "application/json", data = "<item_client>")]
fn item_create(item_client: Json<ItemClient>, shared_client: State<SharedClient>) -> Result<Json<Value>, status::Custom<String>> {

    let item_elastic = match item_client_to_elastic(item_client.into_inner()) {
        Ok(i)  => i,
        Err(err) => return Err(status::Custom(Status::BadRequest, err)),
    };
    let client = shared_client.lock().expect("Could not get elastic client lock");

    println!("{:?}", item_elastic);

    Ok(Json(json!({
        "status": "success",
        "reason": "We always win when creating!"
    })))
}

#[get("/<id>")]
fn item_get(id: ElasticId) -> Json<Value> {
    Json(json!({
        "status": "success",
        "reason": "We always win when getting!"
    }))
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