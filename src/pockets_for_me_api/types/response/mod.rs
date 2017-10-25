use types::ElasticId;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedResponse {
    pub id: ElasticId
}

impl CreatedResponse {
    pub fn new(id: ElasticId) -> CreatedResponse {
        CreatedResponse {
            id: id
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse<T> {
    pub results: Vec<T>,
    pub suggestions: Vec<String>
}
