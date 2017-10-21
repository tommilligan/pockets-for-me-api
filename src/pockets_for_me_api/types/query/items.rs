
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemClient {
    pub category: String,
    pub description: String,
    pub dimensions: [i64; 3],
    pub make: String,
    pub model: String,
    pub version: String,
}
