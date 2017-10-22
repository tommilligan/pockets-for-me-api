extern crate log;

// Rocket
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_derive;
use rocket_contrib::{Json, Value};
use rocket::{Route, State};
use rocket::response::status;
use rocket::http::Status;

// Elastic imports
extern crate elastic;
extern crate elastic_derive;
use elastic::prelude::*;
use elastic::client::SyncClient;


use types::ElasticId;
use types::elastic::items::ItemElastic;
use types::query::items::{ItemClient, ItemSearch};
use generate;

use std::thread;


#[post("/", format = "application/json", data = "<item_client>")]
fn item_create(item_client: Json<ItemClient>, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    log::info!("Creating new item");
    let item_elastic = match ItemElastic::from_item_client(item_client.into_inner()) {
        Ok(i)  => i,
        Err(e) => {
            log::warn!("{:?}", e);
            return status::Custom(Status::BadRequest, Json(json!({"error": e})))
        }
    };

    let response = match elastic_client.document_index(index("items"), id(generate::elastic_id()), item_elastic).send() {
        Ok(r) => r,
        Err(e) => {
            log::error!("{:?}", e);
            return status::Custom(Status::BadGateway, Json(json!({"error": "Database error"})))
        }
    };

    status::Custom(Status::Created, Json(json!({"id": response.id()})))
}

#[get("/<item_id>")]
fn item_get(item_id: ElasticId, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    log::info!("Getting item");
    let response = match elastic_client.document_get::<ItemElastic>(index("items"), id(item_id)).send() {
        Ok(r) => r,
        Err(e) => {
            log::error!("{:?}", e);
            return status::Custom(Status::BadGateway, Json(json!({"error": "Could not get item"})))
        }
    };

    let doc = match response.into_document() {
        Some(d) => d,
        None => {
            log::warn!("Item not found");
            return status::Custom(Status::NotFound, Json(json!({"error": "Item not found"})))
        }
    };
    status::Custom(Status::Ok, Json(json!(doc)))
}

#[get("/?<search_form>")]
fn item_search(search_form: ItemSearch, elastic_client: State<SyncClient>) -> status::Custom<Json<Value>> {
    // Get results in a separate thread while we handle suggestions below
    /*
    let results_handler = thread::spawn(move || {
        log::info!("Searching for item suggestions");
        let query = json!({
            "query": {
                "match": {
                    "name": {
                        "query": &search_form.name,
                        "operator": "and",
                        "fuzziness": "AUTO"
                    }
                }
            }
        });
        log::info!("Searching for item with query; {}", query);
        let response = match elastic_client.search::<ItemElastic>().index("items").body(query).send() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Search failed; {:?}", e);
                return status::Custom(Status::BadGateway, Json(json!({"error": "Could not get item"})))
            }
        };

        let results: Vec<ItemElastic> = response.into_documents().collect();
        results
    });
    */
    let results_handler = thread::spawn(|| {
        log::info!("Searching for item suggestions");
        String::from("hello")
    });


    log::info!("Searching for item name suggestions");
    let query = json!({
        "suggest": {
            "text" : &search_form.name,
            "make" : {
                "term" : {
                    "field" : "name"
                }
            }
        }
    });

    // Need to use a RawRequest (https://docs.rs/elastic/0.20.1/elastic/client/struct.Client.html#raw-request)
    // and submit PR for this!

    log::info!("Searching for item name suggestions with query; {}", query);
    let response = match elastic_client.search().index("items").body(query).send() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Search failed; {:?}", e);
            return status::Custom(Status::BadGateway, Json(json!({"error": "Could not get item"})))
        }
    };
    log::warn!("{:?}", response);

    let suggestions = response;

    let results = results_handler.join().unwrap();
    status::Custom(Status::Ok, Json(json!({
        "results": results,
        "suggestions": ""
    })))
}


pub fn routes() -> Vec<Route> {
    routes![item_create, item_get, item_search]
}
