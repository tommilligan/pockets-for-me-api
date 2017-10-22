
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::{Json, Value};

#[error(404)]
fn not_found() -> status::Custom<Json<Value>> {
    status::Custom(Status::NotFound, Json(json!({
        "error": "Resource was not found"
    })))
}

#[error(404)]
fn service_unavailable() -> status::Custom<Json<Value>> {
    status::Custom(Status::ServiceUnavailable, Json(json!({
        "error": "Service unavailable"
    })))
}