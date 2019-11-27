// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Backends for storing blobs.

*/

use async_trait::async_trait;
use bytes::Bytes;
use std::io::Read;
use std::path::PathBuf;

use crate::digest::DigestData;
use crate::errors::Result;

pub mod filesystem;


/// A trait for getting data chunks asynchronously.
///
/// This trait basically wraps around the API provided by reqwest::Response,
/// but using `async_trait`. The point is to future-proof because we'll likely
/// want to implement other ways of obtaining blobs than just HTTP(S).
#[async_trait]
pub trait AsyncChunks {
    /// Try to get the next chunk of data.
    ///
    /// A return value of None indicates that the stream is finished. See the
    /// documentation for `request::Response::chunk()` for usage guidance.
    async fn get_chunk(&mut self) -> Result<Option<Bytes>>;
}

/// A trait for backends that can store and retrieve blobs.
///
/// I originally implemented this with an associated type for the staging
/// functionality, but we pass around Storage implementors as trait objects,
/// and it seems that you basically can't use associated types with trait
/// objects in a generic fashion. The new API feels less classy but should
/// work just fine.
#[async_trait]
pub trait Storage {
    /// Get a path to a blob, if possible.
    ///
    /// Blobs are identified by their digests. If the blob is not present in
    /// this Storage, or this Storage does not store this blob as a standalone
    /// file on the filesystem, that's OK; `Ok(None)` should be returned.
    fn get_path(&self, digest: &DigestData) -> Result<Option<PathBuf>>;

    /// Open a blob, if possible.
    ///
    /// Blobs are identified by their digests. If the blob is not present in
    /// this Storage, that's OK; `Ok(None)` should be returned.
    fn open(&self, digest: &DigestData) -> Result<Option<Box<dyn Read>>>;

    /// Ingest a new blob.
    ///
    /// The blob is read asynchronously from some source of bytes.
    async fn ingest(
        &mut self,
        mut source: Box<dyn AsyncChunks + Send>,
    ) -> Result<(u64, DigestData)>;
}
