
use super::*;

use types::query::items::ItemClient;

describe! stainless {
    before_each {
        let input_base = ItemClient {
            category: String::from("Phone"),
            make: String::from("Apple"),
            model: String::from("iPhone"),
            version: String::from("4"),
            description: String::from("Apple's last truly reliable phone; the Nokia brick of the smartphone era"),
            dimensions: [59, 116, 10]    
        };
        let _expected_base = ItemElastic {
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
    }

    it "works with a valid phone" {
        let input = ItemClient {
            category: String::from("Phone"),
            ..input_base
        };
        let expected = ItemElastic {
            category: ItemCategories::Phone,
            .._expected_base
        };
        let actual = ItemElastic::from_item_client(input).unwrap();
        assert_eq!(actual, expected);
    }
    it "works with a valid tablet" {
        let input = ItemClient {
            category: String::from("Tablet"),
            ..input_base
        };
        let expected = ItemElastic {
            category: ItemCategories::Tablet,
            .._expected_base
        };
        let actual = ItemElastic::from_item_client(input).unwrap();
        assert_eq!(actual, expected);
    }
    it "fails with an invalid category" {
        let input = ItemClient {
            category: String::from("Spam"),
            ..input_base
        };
        let expected = String::from("Invalid item category 'Spam'");
        let actual = ItemElastic::from_item_client(input).unwrap_err();
        assert_eq!(actual, expected);
    }
}
