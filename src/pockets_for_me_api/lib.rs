// Root crate imports
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![cfg_attr(test, plugin(stainless))]
#![feature(use_extern_macros)]
#![feature(custom_derive)]

// Logging
#[macro_use(log)]
extern crate log;

// Rocket
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

// Elastic imports
extern crate elastic;
#[macro_use]
extern crate elastic_derive;
use elastic::prelude::*;
use elastic::client::SyncClient;
use elastic::http::header::Authorization;

// Internal modules
pub mod types;
pub mod generate;
pub mod routes;
pub mod constants;
pub mod admin;

// Unit tests
#[cfg(test)] mod tests;


extern crate base64;

use base64::encode;


// Client code
pub fn elastic_client() -> Result<SyncClient, elastic::Error> {
    let es_url = constants::elastic_url();
    println!("ES URL; {}", &es_url);
    let creds = es_url.chars().skip(8).take(25).collect::<String>();
    println!("ES Creds; {}", &creds);
    let encoded_creds = encode(&creds);
    println!("Encoded creds; {}", &encoded_creds);
    let builder = SyncClientBuilder::new()
        .base_url(es_url)
        .params(|p| p
            .url_param("pretty", true)
            .header(Authorization(format!("Basic {}", encoded_creds)))
        );

    let client = builder.build()?;
    client.ping().send()?;
    Ok(client)
}

pub fn rocket() -> rocket::Rocket {
    match elastic_client() {
        Ok(client) => {
            // Make sure all indexes exist and are typed correctly
            admin::elastic::ensure_index_mapped_all(&client).unwrap();

            rocket::ignite()
                .mount("/items", routes::items::routes())
                .catch(errors![routes::catchers::not_found])
                .manage(client)
        },
        Err(e) => {
            println!("ERROR setting up elastic client; {:?}", e);
            rocket::ignite()
                .catch(errors![routes::catchers::service_unavailable])
        }
    }
}
