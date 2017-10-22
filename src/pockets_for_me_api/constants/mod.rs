use std::env;

pub fn elastic_url() -> String {
    let default = String::from("http://localhost:9200");
    let key = "BONSAI_URL";
    env::var(key).unwrap_or(default)
}
