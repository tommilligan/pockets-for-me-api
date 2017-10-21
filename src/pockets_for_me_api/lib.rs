// Root crate imports
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![cfg_attr(test, plugin(stainless))]
#![feature(use_extern_macros)]

// Logging
#[macro_use(log)]
extern crate log;

// Rocket
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
use rocket_contrib::{Json, Value};

// Elastic imports
extern crate elastic;
#[macro_use]
extern crate elastic_derive;
use elastic::prelude::*;
use elastic::client::SyncClient;


// Internal modules
pub mod types;
use types::elastic::items::ItemElastic;
pub mod generate;
pub mod routes;

// Unit tests
#[cfg(test)] mod tests;


// Client code


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
        .mount("/items", routes::items::routes("items"))
        .catch(errors![not_found])
        .manage(client)
}
