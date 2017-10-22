use std::env;

pub fn env_heroku() -> bool {
    env::var("BONSAI_URL").is_ok()
}

pub fn elastic_url() -> String {
    let default = String::from("http://localhost:9200");
    let key = "BONSAI_URL";
    env::var(key).unwrap_or(default)
}
