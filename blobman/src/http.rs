// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Retrieving blobs over HTTP(S).
//!
//! With the current state of the Rust/tokio/reqwest async ecosystem, almost
//! everything Just Works, but we need to implement a semi-hacky async trait
//! to be future-proof for reading data from sources that aren't reqwest HTTP
//! response.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest;

use crate::{errors::Result, storage::AsyncChunks};


#[async_trait]
impl AsyncChunks for reqwest::Response {
    async fn get_chunk(&mut self) -> Result<Option<Bytes>> {
        Ok(self.chunk().await?)
    }
}
