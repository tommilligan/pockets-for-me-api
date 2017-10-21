extern crate uuid;

use types::ElasticId;

pub fn elastic_id<'a> () -> ElasticId {
    format!("{}", uuid::Uuid::new_v4().simple())
}
