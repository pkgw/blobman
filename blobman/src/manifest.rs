// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Handling of the manifest of known blobs.

*/

use std::collections::hash_map::{Entry, HashMap};
use std::io as std_io;
use std::io::Read;
use std::path::{Component, PathBuf};
use toml;

use digest::{DigestData, Shim};
use errors::Result;
use io;
use notify::NotificationBackend;
use storage::Storage;


/// The basename used by manifest files.
pub const MANIFEST_STEM: &'static str = ".blobs.toml";
const PARENT_DIR: &'static str = "..";


/// Information about a blob.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlobInfo {
    size: u64,
    sha256: DigestData,
    url: Option<String>,
}


impl BlobInfo {
    /// Ingest a new blob and extract its properties.
    ///
    /// The newly-ingested blob is staged in the storage area *storage*. The
    /// staging happens by calling the function *filler*, which writes data to
    /// a destination that this function hands to it. Upon successful
    /// completion we create a BlobInfo object summarizing the blob contents.
    ///
    /// The somewhat awkward architecture here is because of how we have to
    /// interface with the async, futures-based hyper HTTP library.
    pub fn new_from_ingest<F>(filler: F, storage: &mut Storage) -> Result<Self>
        where F: FnOnce(&mut Shim<Box<std_io::Write>>) -> Result<u64>
    {
        let (cookie, digest, size) = {
            let (sink, cookie) = storage.start_staging()?;
            let mut shim = Shim::new(sink);
            let size = filler(&mut shim)?;
            let (_sink, digest) = shim.finish();
            (cookie, digest, size)
        };
        storage.finish_staging(cookie, &digest)?;

        Ok(Self {
            size: size,
            sha256: digest,
            url: None,
        })
    }

    /// Get the digest associated with this blob.
    pub fn digest<'a>(&'a self) -> &'a DigestData {
        &self.sha256
    }

    /// Set the URL associated with this object.
    pub fn set_url(&mut self, url: &str) {
        self.url = Some(url.to_owned());
    }
}


/// A table of known blobs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    blobs: HashMap<String,BlobInfo>,
}


impl Manifest {
    /// Locate manifest data on the filesystem and load them.
    ///
    /// We first try to load `.blobs.toml`. If that does not exist, we then
    /// try `../.blobs.toml`. We keep on trying higher-level directories until
    /// we reach a filesystem root. If no such file is found, the current
    /// manifest is implicitly empty.
    pub fn find() -> Result<(Self, Option<PathBuf>)> {
        let mut p = PathBuf::from(MANIFEST_STEM);

        loop {
            if let Some(mut f) = ctry!(io::try_open(&p); "error trying to read {}", p.display()) {
                // OK, we've got our hands on a manifest file.
                let mut buf = Vec::<u8>::new();
                f.read_to_end(&mut buf)?;
                let manifest = toml::from_slice(&buf)?;
                return Ok((manifest, Some(p)));
            }

            p.pop(); // "../../.blobs.toml" => "../.."

            // This is the best way I can figure out to determine if we're at
            // a filesystem root:

            let mut at_filesystem_root = true;

            if p.as_os_str().len() == 0 {
                at_filesystem_root = false; // canonicalize() errors out for empty path
            } else {
                for c in ctry!(p.canonicalize(); "error trying to canonicalize path {}", p.display()).components() {
                    if let Component::Normal(_) = c {
                        // We're not at the filesystem root just yet; keep trying parent directories.
                        at_filesystem_root = false;
                        break;
                    }
                }
            }

            if at_filesystem_root {
                // This is OK! It just means that there's no .blobs.toml in
                // our filesystem tree, which we treat as an empty manifest.
                // We'll create the TOML file in the current directory if the
                // manifest is altered.
                return Ok((Self {
                    blobs: HashMap::new(),
                }, None));
            }

            // Try a higher-level parent.
            p.push(PARENT_DIR);
            p.push(MANIFEST_STEM);
        }
    }


    /// Look up information for the named blob.
    pub fn lookup<'a>(&'a self, name: &str) -> Option<&'a BlobInfo> {
        self.blobs.get(name)
    }


    /// Register a new blob with the manifest.
    ///
    /// If a blob under the same name was already known, the old information
    /// is replaced.
    pub fn insert_or_update<B: NotificationBackend>(&mut self, name: &str, binfo: BlobInfo, nbe: &mut B) {
        let e = self.blobs.entry(name.to_owned());

        match e {
            Entry::Occupied(mut oe) => {
                if oe.get() != &binfo {
                    bm_note!(nbe, "updating entry for {}", name);
                } else {
                    bm_note!(nbe, "entry for {} is unchanged", name);
                }
                oe.insert(binfo);
            },
            Entry::Vacant(ve) => {
                ve.insert(binfo);
            }
        }
    }
}
