// Copyright 2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Collections of blobs.
//!
//! For (de)serialization, we want to convert between BlobIds and DigestDatas.
//! I can't find a better way to implement this than to have two variants of
//! the serialization structs with conversion routines that use a DigestTable.
//! It seems like there should be some way to incorporate the context of the
//! DigestTable into the serialization process, but I can't figure out how to
//! do so short of implementing my own serializer, and even then I'm not sure
//! if that would do the trick given serde's data model.

use serde::{Deserialize, Serialize};
use toml::value::Table;

use crate::{
    blobs::{BlobId, DigestTable},
    ctry,
    digest::DigestData,
    errors::Result,
    retrieval::{AnyRetrieval, Retrieval},
    SessionServices,
};

/// A collection of blobs.
#[derive(Debug)]
pub struct Collection {
    pub(crate) name: String,
    keys: Vec<String>,
    pub(crate) retrieval: Box<dyn Retrieval + Send>,
    items: Vec<Item>,
    metadata: Table,
}

impl Collection {
    /// Create a new, empty collection.
    pub fn new(name: &str, retrieval: Box<dyn Retrieval + Send>) -> Self {
        Collection {
            name: name.to_owned(),
            keys: Vec::new(),
            retrieval,
            items: Vec::new(),
            metadata: Table::new(),
        }
    }

    /// Change the set of keys used to uniquely identify items in this collection.
    pub fn set_keys<T>(&mut self, keys: T)
    where
        T: IntoIterator,
        T::Item: ToString,
    {
        self.keys = keys.into_iter().map(|v| v.to_string()).collect();
    }

    /// Insert a new item into the collection.
    pub async fn insert_item<'a>(&mut self, item_spec: Table, services: &mut SessionServices<'a>) -> Result<BlobId> {
        // TODO: check for item uniqueness / newness

        let mut storage = ctry!(services.get_storage(); "cannot open storage backend");

        let stream = self.retrieval.retrieve(services, &item_spec).await?;
        let (size, digest) = storage.ingest(stream).await?;
        let id = services.digest_table.digest_to_id(&digest);
        let item = Item {
            id,
            size,
            extra: item_spec,
        };

        self.items.push(item);
        Ok(id)
    }

    /// Clone this object into a serializable version of itself.
    pub(crate) fn clone_serde(&self, dtable: &DigestTable) -> SerdeCollection {
        let item = self.items.iter().map(|i| i.clone_serde(dtable)).collect();

        SerdeCollection {
            name: self.name.clone(),
            keys: self.keys.clone(),
            retrieval: self.retrieval.clone_any(),
            item,
            metadata: self.metadata.clone(),
        }
    }
}

/// (De)serializable version of Item.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SerdeCollection {
    name: String,
    keys: Vec<String>,
    retrieval: AnyRetrieval,
    item: Vec<SerdeItem>,
    metadata: Table,
}

impl SerdeCollection {
    pub(crate) fn into_runtime(mut self, dtable: &mut DigestTable) -> Collection {
        let retr = self.retrieval.boxify();

        let items = self
            .item
            .drain(..)
            .map(|i| i.into_runtime(dtable))
            .collect();

        Collection {
            name: self.name,
            keys: self.keys,
            retrieval: retr,
            items: items,
            metadata: self.metadata,
        }
    }
}

/// An item within a blob collection.
#[derive(Debug)]
pub struct Item {
    /// The runtime BlobId of this item.
    pub id: BlobId,

    /// The size of the blob data, in bytes.
    pub size: u64,

    /// Other user-defined key/value pairs for this item.
    pub extra: Table,
}

impl Item {
    /// Clone this object into a serializable version of itself.
    pub(crate) fn clone_serde(&self, dtable: &DigestTable) -> SerdeItem {
        let digest = dtable.id_to_digest(self.id).clone();

        SerdeItem {
            digest,
            size: self.size,
            extra: self.extra.clone(),
        }
    }
}

/// (De)serializable version of Item.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SerdeItem {
    digest: DigestData,
    size: u64,

    #[serde(flatten)]
    extra: Table,
}

impl SerdeItem {
    fn into_runtime(self, dtable: &mut DigestTable) -> Item {
        let id = dtable.digest_to_id(&self.digest);
        Item {
            id,
            size: self.size,
            extra: self.extra,
        }
    }
}
