// Copyright 2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Low-level management of blobs.

use radix_trie::Trie;

use crate::digest::DigestData;

/// A blob identifier.
///
/// Each blob is uniquely defined by its contents. This blob identifier is a
/// unique reference to a blob valid during the execution of this program.
/// There is a one-to-one mapping between BlobIds and DigestData values,
/// accessible through a DigestTable. Unlike DigestData, BlobId is trivially
/// Copy-able.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobId(usize);

/// A table mapping BlobIds to DigestData.
///
/// The table is append-only: it never forgets about a digest/BlobId pair once
/// it is learned.
pub struct DigestTable {
    digests: Vec<DigestData>,
    digest_to_id: Trie<DigestData, BlobId>,
}

impl DigestTable {
    /// Create a new, empty DigestTable.
    pub fn new() -> Self {
        DigestTable {
            digests: Vec::new(),
            digest_to_id: Trie::new(),
        }
    }

    /// Given a BlobId, get its corresponding DigestData.
    ///
    /// This operation can never fail, because a BlobId can only be created by
    /// registering DigestData with the table.
    pub fn id_to_digest(&self, id: BlobId) -> &DigestData {
        &self.digests[id.0]
    }

    /// Given DigestData, get a BlobId, possibly allocating a new one.
    pub fn digest_to_id(&mut self, digest: &DigestData) -> BlobId {
        if let Some(id) = self.digest_to_id.get(digest) {
            return *id;
        }

        let id = BlobId(self.digests.len());
        self.digests.push(digest.clone());
        self.digest_to_id.insert(self.digests[id.0], id);
        id
    }
}
