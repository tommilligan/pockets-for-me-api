use types::ElasticId;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedResponse {
    pub id: ElasticId
}
