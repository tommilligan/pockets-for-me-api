// Rocket imports
#![feature(plugin)]
#![plugin(rocket_codegen)]

// Logging

#![feature(use_extern_macros)]
#[macro_use(log)] extern crate log;


extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use rocket_contrib::{Json, Value};

use rocket::State;

use rocket::response::status;
use rocket::http::Status;

// Elastic imports

extern crate elastic;
#[macro_use]
extern crate elastic_derive;

use elastic::prelude::*;
use elastic::client::SyncClient;


// Other imports
extern crate uuid;
use uuid::Uuid;


// Modules

pub mod types;
use types::ElasticId;
use types::items::{ItemClient, ItemElastic, ItemCategories};


// Client code

fn new_elastic_id<'a> () -> Id<'a> {
    id(format!("{}", Uuid::new_v4().simple()))
}

fn item_name(make: &str, model: &str, version: &str) -> String {
    format!("{} {} ({})", model, version, make)
}

fn item_client_to_elastic(item_client: ItemClient) -> Result<ItemElastic, String> {
    log::info!("Converting client submitted item to Elastic document");
    let item = item_client;

    // Store dimensions in descending order
    let mut sorted_dimensions = item.dimensions.clone();
    sorted_dimensions.sort();

    // Parses the category given and make sure it is valid
    let category_enum = item.category.parse::<ItemCategories>()?;

    // Compute a name for this device
    let name = item_name(&item.make, &item.model, &item.version);

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
fn item_create(item_client: Json<ItemClient>, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    log::info!("Creating new item");
    let item_elastic = match item_client_to_elastic(item_client.into_inner()) {
        Ok(i)  => i,
        Err(e) => return status::Custom(Status::BadRequest, Json(json!({"error": e}))),
    };

    let response = match elastic_client.document_index(index("items"), id(new_elastic_id()), item_elastic).send() {
        Ok(r) => r,
        Err(_e) => return status::Custom(Status::BadGateway, Json(json!({"error": "Database error"}))),
    };

    status::Custom(Status::Created, Json(json!({"id": response.id()})))
}

#[get("/<item_id>")]
fn item_get(item_id: ElasticId, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    log::info!("Getting item");
    let response = match elastic_client.document_get::<ItemElastic>(index("items"), id(item_id)).send() {
        Ok(r) => r,
        Err(_e) => return status::Custom(Status::BadGateway, Json(json!({"error": "Could not get item"}))),
    };

    let doc = match response.into_document() {
        Some(d) => d,
        None => return status::Custom(Status::NotFound, Json(json!({"error": "Item does not exist"}))),
    };
    status::Custom(Status::Ok, Json(json!(doc)))
}


#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "error": "Resource was not found."
    }))
}

fn rocket() -> rocket::Rocket {
    let builder = SyncClientBuilder::new()
        .base_url("http://localhost:9200")
        .params(|p| p
            .url_param("pretty", true)
        );

    let client = builder.build().expect("Could not build elastic client");

    // Make sure indexes are typed correctly
    client.document_put_mapping::<ItemElastic>(index("items"))
        .send().expect("Items index already had a conflicting mapping");

    rocket::ignite()
        .mount("/items", routes![item_create, item_get])
        .catch(errors![not_found])
        .manage(client)
}

fn main() {
    rocket().launch();
}