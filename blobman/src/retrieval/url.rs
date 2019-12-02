// Copyright 2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Retrieving items in a collection where each one is tagged with a URL.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use toml::{self, value::Table};

use crate::{
    err_msg,
    errors::Result,
    SessionServices,
    storage::AsyncChunks,
};

use super::{AnyRetrieval, Retrieval};

/// Retrieving items where each is tagged with a URL.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UrlRetrieval {}

impl UrlRetrieval {
    /// Create a new UrlRetrieval item.
    pub fn new() -> Self {
        UrlRetrieval {}
    }
}

#[async_trait]
impl Retrieval for UrlRetrieval {
    async fn retrieve<'a>(&mut self, services: &mut SessionServices<'a>, item_spec: &Table) -> Result<Box<dyn AsyncChunks + Send>> {
        if let Some(toml::Value::String(ref url)) = item_spec.get("url") {
            Ok(Box::new(services.get_url(url).await?) as Box<dyn AsyncChunks + Send>)
        } else{
            // for later: better indication of which item we're talking about?
            err_msg!("item {:?} is missing a string \"url\" field", item_spec)
        }
    }

    fn clone_any(&self) -> AnyRetrieval {
        AnyRetrieval::Url(self.clone())
    }
}
