// Rocket imports
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![cfg_attr(test, plugin(stainless))]

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


// Modules

pub mod types;
use types::ElasticId;
use types::elastic::items::ItemElastic;
use types::query::items::ItemClient;

pub mod generate;

// Client code

#[post("/", format = "application/json", data = "<item_client>")]
fn item_create(item_client: Json<ItemClient>, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    log::info!("Creating new item");
    let item_elastic = match ItemElastic::from_item_client(item_client.into_inner()) {
        Ok(i)  => i,
        Err(e) => return status::Custom(Status::BadRequest, Json(json!({"error": e}))),
    };

    let response = match elastic_client.document_index(index("items"), id(generate::elastic_id()), item_elastic).send() {
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

pub fn elastic_client() -> SyncClient {
    let builder = SyncClientBuilder::new()
        .base_url("http://localhost:9200")
        .params(|p| p
            .url_param("pretty", true)
        );

    let client = builder.build().expect("Could not build elastic client");
    client
}

pub fn rocket() -> rocket::Rocket {
    let client = elastic_client();

    // Make sure indexes are typed correctly
    client.document_put_mapping::<ItemElastic>(index("items"))
        .send().expect("Items index already had a conflicting mapping");

    rocket::ignite()
        .mount("/items", routes![item_create, item_get])
        .catch(errors![not_found])
        .manage(client)
}
