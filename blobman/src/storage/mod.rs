// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Backends for storing blobs.

*/

use std::io::Write;

use digest::DigestData;
use errors::Result;


pub mod filesystem;


/// A trait for backends that can store and retrieve blobs.
pub trait Storage<'a> {
    /// This type is used to ingest data for new blobs.
    type Stager: 'a + Write + StagerOps;

    /// Start staging a new file.
    ///
    /// Staging is performed by creating a "Stager" object. Blob data is
    /// written to the Stager, and then a wrap-up function is called to
    /// complete the transaction.
    fn new_stager(&'a mut self) -> Result<Self::Stager>;
}


/// A trait for Stager objects
pub trait StagerOps {
    /// Called when all blob data have been processed.
    ///
    /// An error should be returned if there was a problem completing
    /// the staging process.
    fn finish(mut self, digest: &DigestData) -> Result<()>;
}
