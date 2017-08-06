// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Handling of the manifest of known blobs.

*/

use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, PathBuf};
use toml;

use digest::DigestData;
use errors::Result;
use io;


const MANIFEST_STEM: &'static str = ".blobs.toml";
const PARENT_DIR: &'static str = "..";


/// Information about a blob.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlobInfo {
    name: String,
    size: u64,
    sha256: DigestData,
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
}
