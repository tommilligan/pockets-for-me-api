#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

extern crate serde_json;
extern crate pockets_for_me_api;
extern crate rocket;
extern crate elastic;

use pockets_for_me_api::{elastic_client, rocket};
use rocket::local::Client;
use rocket::http::{Status, ContentType};

use elastic::prelude::*;
use elastic::error::Error::Api;
use elastic::error::ApiError::IndexNotFound;

use pockets_for_me_api::types::response::CreatedResponse;
use pockets_for_me_api::types::elastic::items::ItemElastic;

describe! stainless {
    before_each {
        let es = elastic_client();
        let response = match es.index_delete(index("items")).send() {
            Ok(r) => (),
            Err(Api(IndexNotFound{ index: _i })) => (),
            Err(e) => panic!(format!("Could not delete elastic indexes: {}", e)),
        };
        es.index_create(index("items")).send().expect("Could not create items index");        
        es.document_put_mapping::<ItemElastic>(index("items"))
            .send().expect("Items index already had a conflicting mapping");

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

        // Check we got an id back, and use t to get the created object back
        let created_id = j.id;
        let endpoint = format!("/items/{}", created_id);

        // Check that the item exists with the correct contents
        let mut res = client.get(endpoint).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body().unwrap().into_string().unwrap();
        let j: ItemElastic = serde_json::from_str(&body).unwrap();

        assert_eq!(j.model, "iPhone");
        assert_eq!(j.dimension_x, 116);
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