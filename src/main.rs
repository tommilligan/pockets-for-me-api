#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use rocket_contrib::{Json, Value};

type ElasticId = String;

#[derive(Serialize, Deserialize)]
struct Item {
    category: String,
    make: String,
    model: String,
    version: String,
    name: String,
    description: String,
    dimensions: [i32; 3]
}

#[post("/", format = "application/json", data = "<item>")]
fn item_create(item: Json<Item>) -> Json<Value> {
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
    rocket::ignite()
        .mount("/items", routes![item_create, item_get])
        .catch(errors![not_found])
}

fn main() {
    rocket().launch();
}