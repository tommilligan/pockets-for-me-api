extern crate log;

// Rocket
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
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
use types::response::{CreatedResponse, SearchResponse};
use types::elastic::items::ItemElastic;
use types::query::items::{ItemClient, ItemSearch};
use generate;

use std::thread;

use types::SuggestResponse;

extern crate itertools;
use self::itertools::Itertools;

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

    status::Custom(Status::Created, Json(json!(CreatedResponse::new(String::from(response.id())))))
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
    // RESULTS
    // TODO Get results in a separate thread while we handle suggestions below
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
    
    // SUGGESTIONS
    let query = json!({
        "text" : &search_form.name,
        "qux" : {
            "term" : {
                "field" : "name"
            }
        }
    });
    log::info!("Searching for item name suggestions with query; {}", query);
    let req = SuggestRequest::for_index(index("items"), query);
    let response = match elastic_client.request(req).send() {
        Ok(r) => {
            match r.into_response::<SuggestResponse>() {
                Ok(s) => s,
                Err(e) => {
                    log::error!("SuggestResponse deserialisation failed; {:?}", e);
                    return status::Custom(Status::InternalServerError, Json(json!({"error": "Could not read item suggestions"})))
                }
            }
        },
        Err(e) => {
            log::error!("Search failed; {:?}", e);
            return status::Custom(Status::BadGateway, Json(json!({"error": "Could not get item suggestions"})))
        }
    };
    let suggestees = response.inner();
    log::info!("Raw suggestions; {:?}", suggestees);

    let mut suggested_names: Vec<String> = Vec::new();
    let suggestees_length = suggestees.len();

    // We need at least on suggestee to provide suggestions
    if suggestees_length > 0 {
        // Append final word suggestions
        let last_index = suggestees_length - 1;
        let last_suggestee = &suggestees[last_index];
        let stem: String = suggestees[0..last_index]
                .iter()
                .map(|s| s.text())
                .join(" ");
        for suggestion in last_suggestee.suggestions() {
            let n = format!("{} {}", &stem, suggestion.text());
            suggested_names.push(n);
        }

        // Append best suggestion
        let n: String = suggestees.iter().map(|suggestee| {
            let suggestions = suggestee.suggestions();
            let word: &str = match suggestions.len() {
                0 => suggestee.text(),
                _ => suggestions[0].text()
            };
            word
        }).join(" ");
        suggested_names.push(n);
    }

    status::Custom(Status::Ok, Json(json!(SearchResponse::<ItemElastic> {
        results: results,
        suggestions: suggested_names
    })))
}


pub fn routes() -> Vec<Route> {
    routes![item_create, item_get, item_search]
}
