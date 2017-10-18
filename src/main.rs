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

#[derive(Serialize, Deserialize)]
enum ItemCategories {
    Phone,
    Tablet
}
impl KeywordFieldType<DefaultKeywordMapping> for ItemCategories {}
impl FromStr for ItemCategories {
    type Err = ();

    fn from_str(s: &str) -> Result<ItemCategories, ()> {
        match s {
            "phone" => Ok(ItemCategories::Phone),
            "tablet" => Ok(ItemCategories::Tablet),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, ElasticType)]
struct ItemDimensions {
    x: i32,
    y: i32,
    z: i32
}

#[derive(Serialize, Deserialize, ElasticType)]
struct ItemElastic {
    category: ItemCategories,
    make: String,
    model: String,
    version: String,
    name: String,
    description: String,
    dimensions: ItemDimensions
}

#[derive(Serialize, Deserialize)]
struct ItemClient {
    category: String,
    make: String,
    model: String,
    version: String,
    name: String,
    description: String,
    dimensions: [i32; 3]
}

#[post("/", format = "application/json", data = "<item>")]
fn item_create(item: Json<ItemClient>, shared_client: State<SharedClient>) -> Json<Value> {

    let client = shared_client.lock().expect("Could not get elastic client lock");


    Json(json!({
        "status": "success",
        "reason": "We always win when creating!"
    }))
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