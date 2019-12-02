// Copyright 2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Mechanisms for retrieving blobs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use toml::value::Table;

use crate::{
    errors::Result,
    SessionServices,
    storage::AsyncChunks,
};

pub mod url;

/// Something that knows how to retrieve the contents of a blob
#[async_trait]
pub trait Retrieval: Debug {
    /// Start obtaining the contents of a blob.
    ///
    /// This function asynchronously returns a boxed AsyncChunks object, which
    /// in turn can asynchronously retrieve chunks of blob content
    /// progressively.
    async fn retrieve<'a>(&mut self, services: &mut SessionServices<'a>, item_spec: &Table) -> Result<Box<dyn AsyncChunks + Send>>;

    /// Clone this object into an AnyRetrieval value
    fn clone_any(&self) -> AnyRetrieval;
}


/// An enumeration of all retrieval types recognized by this crate.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnyRetrieval {
    /// Retrieval performed using per-item "url" records.
    Url(url::UrlRetrieval),
}

impl AnyRetrieval {
    pub(crate) fn boxify(self) -> Box<dyn Retrieval + Send> {
        match self {
            AnyRetrieval::Url(retr) => Box::new(retr),
        }
    }
}
