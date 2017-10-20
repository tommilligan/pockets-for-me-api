
use rocket;
use rocket::local::Client;
use rocket::http::{Status, ContentType};

extern crate serde_json;



#[derive(Debug, Serialize, Deserialize)]
enum ItemCategories {
    Phone,
    Tablet
}

#[derive(Debug, Serialize, Deserialize)]
struct ItemElastic {
    category: ItemCategories,
    description: String,
    dimension_x: i32,
    dimension_y: i32,
    dimension_z: i32,
    make: String,
    model: String,
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatedResponse {
    id: String
}

#[test]
fn post_get() {
    let client = Client::new(rocket()).unwrap();

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

#[test]
fn bad_get() {
    let client = Client::new(rocket()).unwrap();
    let res = client.get("/items/spam").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), Status::NotFound);
}

#[test]
fn bad_post_category() {
    let client = Client::new(rocket()).unwrap();

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
