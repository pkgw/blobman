// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Backends for storing blobs.

*/

use std::io::{Read, Write};
use std::path::PathBuf;

use digest::DigestData;
use errors::Result;


pub mod filesystem;


/// An type alias referring to a particular staging job.
pub type StagingCookie = usize;

/// A trait for backends that can store and retrieve blobs.
///
/// I originally implemented this with an associated type for the staging
/// functionality, but we pass around Storage implementors as trait objects,
/// and it seems that you basically can't use associated types with trait
/// objects in a generic fashion. The new API feels less classy but should
/// work just fine.
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
    fn open(&self, digest: &DigestData) -> Result<Option<Box<Read>>>;

    /// Start staging a new file.
    ///
    /// Staging is performed by creating a "stager" object. Blob data is
    /// written to the it, and then a wrap-up function is called to complete
    /// the transaction.
    fn start_staging(&mut self) -> Result<(Box<Write>, StagingCookie)>;

    /// Called when all blob data have been processed.
    ///
    /// An error should be returned if there was a problem completing
    /// the staging process.
    fn finish_staging(&mut self, cookie: StagingCookie, digest: &DigestData) -> Result<()>;
}
