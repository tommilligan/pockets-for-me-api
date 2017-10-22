
use super::*;

use types::query::items::ItemClient;

#[test]
fn elastic_item_from_client_valid() {
    let input = ItemClient {
        category: String::from("Phone"),
        make: String::from("Apple"),
        model: String::from("iPhone"),
        version: String::from("4"),
        description: String::from("Apple's last truly reliable phone; the Nokia brick of the smartphone era"),
        dimensions: [59, 116, 10]    
    };
    let expected = ItemElastic {
        category: ItemCategories::Phone,
        description: String::from("Apple's last truly reliable phone; the Nokia brick of the smartphone era"),
        dimension_x: 116,
        dimension_y: 59,
        dimension_z: 10,
        make: String::from("Apple"),
        model: String::from("iPhone"),
        name: String::from("iPhone 4 (Apple)"),
        version: String::from("4"),
    };
    let actual = ItemElastic::from_item_client(input).unwrap();
    assert_eq!(actual, expected);
}
