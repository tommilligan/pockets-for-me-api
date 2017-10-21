use elastic::prelude::*;
use std::str::FromStr;

use super::super::query::items::ItemClient;

extern crate log;

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

impl ItemElastic {
    pub fn from_item_client(item_client: ItemClient) -> Result<ItemElastic, String> {
        log::info!("Converting client submitted item to Elastic document");
        let item = item_client;

        // Store dimensions in descending order
        let mut sorted_dimensions = item.dimensions.clone();
        sorted_dimensions.sort();

        // Parses the category given and make sure it is valid
        let category_enum = item.category.parse::<ItemCategories>()?;

        // Compute a name for this device
        let name = item.name();

        let item_elastic = ItemElastic {
            category: category_enum,
            description: item.description,
            dimension_x: sorted_dimensions[2],
            dimension_y: sorted_dimensions[1],
            dimension_z: sorted_dimensions[0],
            make: item.make,
            model: item.model,
            name: name,
            version: item.version,
        };
        Ok(item_elastic)
    }
}