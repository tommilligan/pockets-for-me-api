extern crate elastic;
use elastic::prelude::*;
use elastic::error::Error::Api;
use elastic::error::ApiError::{IndexNotFound, IndexAlreadyExists};

use types::elastic::items::ItemElastic;


pub fn ensure_index_deleted_items(client: &SyncClient) -> Result<(), elastic::error::Error> {
    match client.index_delete(index("items")).send() {
        Ok(_r) => Ok(()),
        Err(Api(IndexNotFound{ index: _i })) => Ok(()),
        Err(e) => Err(e),
    }?;
    Ok(())
}

pub fn ensure_index_mapped_items(client: &SyncClient) -> Result<(), elastic::error::Error> {
    let index_name = "items";
    let body = json!({
        "settings": {
            "index": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            }
        }
    });
    match client.index_create(index(index_name)).body(body.to_string()).send() {
        Ok(_r) => Ok(()),
        Err(Api(IndexAlreadyExists{ index: _i })) => Ok(()),
        Err(e) => Err(e),
    }?;
    client.document_put_mapping::<ItemElastic>(index(index_name)).send()?;
    Ok(())
}

pub fn ensure_index_mapped_all(client: &SyncClient) -> Result<(), elastic::error::Error> {
    ensure_index_mapped_items(client)?;
    Ok(())
}