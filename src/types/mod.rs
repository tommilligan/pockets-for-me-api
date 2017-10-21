
pub type ElasticId = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedResponse {
    pub id: ElasticId
}

pub mod items;
