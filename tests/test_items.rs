#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

extern crate serde_json;
extern crate pockets_for_me_api;
extern crate rocket;
extern crate elastic;

use pockets_for_me_api::{elastic_client, rocket};
use rocket::local::Client;
use rocket::http::{Status, ContentType};

use pockets_for_me_api::types::response::{CreatedResponse, SearchResponse};
use pockets_for_me_api::types::elastic::items::ItemElastic;
use pockets_for_me_api::admin::elastic::ensure_index_deleted_items;

use std::{thread, time};

fn pause(duration: u64) -> () {
    thread::sleep(time::Duration::from_millis(duration));
}

describe! stainless {
    before_each {
        let es = elastic_client().unwrap();
        ensure_index_deleted_items(&es).unwrap();
        let client = Client::new(rocket()).unwrap();
    }

    it "post then get an item" {
        // Add a new item and get its id
        let mut res = client.post("/items")
            .header(ContentType::JSON)
            .body(r#"{
                "category": "Phone",
                "make": "Apple",
                "model": "iPhone",
                "version": "4",
                "description": "Apple's last truly reliable phone; the Nokia brick of the smartphone era",
                "dimensions": [59, 116, 10]
            }"#)
            .dispatch();

        assert_eq!(res.status(), Status::Created);
        let body = res.body().unwrap().into_string().unwrap();
        println!("{}", &body);
        let j: CreatedResponse = serde_json::from_str(&body).unwrap();
        let created_id = j.id;

        // Pause to check index has caught up
        pause(1000);

        // Check that the item exists with the correct contents
        let endpoint = format!("/items/{}", created_id);
        let mut res = client.get(endpoint).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        let j: ItemElastic = serde_json::from_str(&body).unwrap();

        assert_eq!(j.model, "iPhone");
        assert_eq!(j.dimension_x, 116);
    }
    
    it "post then search an item" {
        // Add a new item
        let res = client.post("/items")
            .header(ContentType::JSON)
            .body(r#"{
                "category": "Phone",
                "make": "Apple",
                "model": "iPhone",
                "version": "4",
                "description": "Apple's last truly reliable phone; the Nokia brick of the smartphone era",
                "dimensions": [59, 116, 10]
            }"#)
            .dispatch();

        assert_eq!(res.status(), Status::Created);

        // Pause to check index has caught up
        pause(1000);

        // Check we can find the item in a search (fuzzy)
        let endpoint = "/items?name=appl iphon";
        let mut res = client.get(endpoint).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        let j: SearchResponse<ItemElastic> = serde_json::from_str(&body).unwrap();
        assert_eq!(j.results[0].model, "iPhone");
        assert_eq!(j.results[0].dimension_x, 116);
        assert_eq!(j.suggestions[0], "appl iphone");
    }

    it "fails to get an item that does not exist" {
        let res = client.get("/items/spam").header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }

    it "rejects an item post with an invalid category" {
        // Add a new item with invalid category
        let res = client.post("/items")
            .header(ContentType::JSON)
            .body(r#"{
                "category": "phine",
                "make": "Apple",
                "model": "iPhone",
                "version": "4",
                "description": "Apple's last truly reliable phone; the Nokia brick of the smartphone era",
                "dimensions": [59, 116, 10]
            }"#)
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);
    }

    after_each {
        // End the test.
        ()
    }
}