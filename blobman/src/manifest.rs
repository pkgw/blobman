// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Handling of the manifest of known blobs.

use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, PathBuf};
use std::result::Result as StdResult;
use toml;

use crate::{
    blobs::DigestTable,
    collection::{Collection, SerdeCollection},
    ctry,
    errors::Result,
    io,
};

/// The basename used by manifest files.
pub const MANIFEST_STEM: &'static str = ".blobs.toml";
const PARENT_DIR: &'static str = "..";

/// A table of blob collections.
#[derive(Debug)]
pub struct Manifest {
    pub(crate) collections: HashMap<String, Collection>,
}

impl Manifest {
    /// Locate manifest data on the filesystem and load them.
    ///
    /// We first try to load `.blobs.toml`. If that does not exist, we then
    /// try `../.blobs.toml`. We keep on trying higher-level directories until
    /// we reach a filesystem root. If no such file is found, the current
    /// manifest is implicitly empty.
    pub fn find(dtable: &mut DigestTable) -> Result<(Self, Option<PathBuf>)> {
        let mut p = PathBuf::from(MANIFEST_STEM);

        loop {
            if let Some(mut f) = ctry!(io::try_open(&p); "error trying to read {}", p.display()) {
                // OK, we've got our hands on a manifest file.
                let mut buf = Vec::<u8>::new();
                f.read_to_end(&mut buf)?;
                let manifest: SerdeManifest = toml::from_slice(&buf)?;
                let manifest = manifest.into_runtime(dtable);
                return Ok((manifest, Some(p)));
            }

            p.pop(); // "../../.blobs.toml" => "../.."

            // This is the best way I can figure out to determine if we're at
            // a filesystem root:

            let mut at_filesystem_root = true;

            if p.as_os_str().len() == 0 {
                at_filesystem_root = false; // canonicalize() errors out for empty path
            } else {
                for c in
                    ctry!(p.canonicalize(); "error trying to canonicalize path {}", p.display())
                        .components()
                {
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
                return Ok((
                    Self {
                        collections: HashMap::new(),
                    },
                    None,
                ));
            }

            // Try a higher-level parent.
            p.push(PARENT_DIR);
            p.push(MANIFEST_STEM);
        }
    }

    /// Clone this object into a serializable version of itself.
    ///
    /// I'm not happy with this model since we shouldn't have to clone all of
    /// our data, but the only way I can see to get one of these objects into
    /// a Serde-friendly state without cloning would be to implement a *third*
    /// variant of the struct, like SerdeManifest with borrows. That seems
    /// like just too much.
    pub(crate) fn clone_serde(&self, dtable: &DigestTable) -> SerdeManifest {
        let collections = self
            .collections
            .iter()
            .map(|(k, v)| (k.clone(), v.clone_serde(dtable)))
            .collect();

        SerdeManifest { collections }
    }
}

/// (De)serializable version of Manifest.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SerdeManifest {
    #[serde(serialize_with = "serialize_map_sorted")]
    collections: HashMap<String, SerdeCollection>,
}

/// From StackOverflow: https://stackoverflow.com/a/42723390/3760486
fn serialize_map_sorted<S>(
    value: &HashMap<String, SerdeCollection>,
    serializer: S,
) -> StdResult<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl SerdeManifest {
    fn into_runtime(mut self, dtable: &mut DigestTable) -> Manifest {
        let colls = self
            .collections
            .drain()
            .map(|(k, v)| (k, v.into_runtime(dtable)))
            .collect();

        Manifest { collections: colls }
    }
}
