use elastic::prelude::*;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemCategories {
    Phone,
    Tablet
}
impl KeywordFieldType<DefaultKeywordMapping> for ItemCategories {}
impl FromStr for ItemCategories {
    type Err = String;

    fn from_str(s: &str) -> Result<ItemCategories, String> {
        match s {
            "Phone" => Ok(ItemCategories::Phone),
            "Tablet" => Ok(ItemCategories::Tablet),
            _ => Err(format!("Invalid item category '{}'", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ElasticType)]
pub struct ItemElastic {
    pub category: ItemCategories,
    pub description: String,
    pub dimension_x: i64,
    pub dimension_y: i64,
    pub dimension_z: i64,
    pub make: String,
    pub model: String,
    pub name: String,
    pub version: String,
}
