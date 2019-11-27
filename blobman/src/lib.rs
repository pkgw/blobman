// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
This crate provides a framework for managing binary “blobs” of data.

A blob is just a file whose contents the framework does not care about, except
that blobman’s job is to ensure that the contents are exactly what the caller
expects.

*/

#![recursion_limit = "1024"] // "error_chain can recurse deeply"
#![deny(missing_docs)]

pub mod config;
pub mod digest;
pub mod errors;
pub mod http;
pub mod io;
pub mod manifest;
pub mod notify;
pub mod storage;

use reqwest::{self, Url};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{
    errors::{Error, Result},
    notify::NotificationBackend,
};

const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo {
    name: "blobman",
    author: "BlobmanProject",
};

/// Different ways that we can behave when ingesting a new blob, depending on
/// whether a blob of the same name or same contents already exists.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IngestMode {
    /// Ingest the new blob fully. If there is an existing blob of the same
    /// name, update the record for that blob to refer to the new contents.
    Update,

    /// If a blob of the same name already exists, don't bother ingesting the
    /// new one, without even checking whether the candidate new blob might
    /// have different contents. If no blob of the same name already exists,
    /// ingest it.
    TrustExisting,
}

impl IngestMode {
    /// Return a list of valid stringifications of the IngestMode type. The
    /// purpose of this function is to assist the CLI in parsing command-line
    /// arguments that map to IngestMode values.
    pub fn stringifications() -> &'static [&'static str] {
        static S: &'static [&str] = &["update", "trust"];
        S
    }
}

impl FromStr for IngestMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "update" {
            Ok(IngestMode::Update)
        } else if s == "trust" {
            Ok(IngestMode::TrustExisting)
        } else {
            err_msg!("unrecognized ingestion mode \"{}\"", s)
        }
    }
}

/// A session in which we do stuff.
pub struct Session<'a> {
    config: &'a config::UserConfig,
    nbe: &'a mut dyn notify::NotificationBackend,
    manifest: manifest::Manifest,
    manifest_path: Option<PathBuf>,
    manifest_modified: bool,
    http_client: reqwest::Client,
}

impl<'a> Session<'a> {
    /// Create and return a new Session.
    pub fn new(config: &'a config::UserConfig, nbe: &'a mut dyn NotificationBackend) -> Result<Self> {
        let (manifest, manifest_path) = manifest::Manifest::find()?;

        Ok(Self {
            config: config,
            nbe: nbe,
            manifest: manifest,
            manifest_path: manifest_path,
            manifest_modified: false,
            http_client: reqwest::Client::new(),
        })
    }

    /// Get a storage backend for this session.
    ///
    /// TODO: Maybe we'll one day have multiple backends and some fancy way to
    /// decide which one to use. For now we just use one. We do, however, use
    /// a trait object since we envision have runtime-configurable backends
    /// here.
    pub fn get_storage(&mut self) -> Result<Box<dyn storage::Storage>> {
        self.config.get_storage(self.nbe)
    }

    /// Fetch a blob from a URL and ingest it.
    pub async fn ingest_from_url(
        &mut self,
        mode: IngestMode,
        url: &str,
        name: Option<&str>,
    ) -> Result<()> {
        let parsed = Url::parse(url)?;
        let file_name = match name {
            Some(n) => n,
            None => match parsed.path().split("/").last() {
                None | Some("") => {
                    return err_msg!("cannot extract a filename from the URL {}", url);
                }
                Some(s) => s,
            },
        };

        let mut storage = ctry!(self.get_storage(); "cannot open storage backend");

        if let IngestMode::TrustExisting = mode {
            if self.manifest.lookup(file_name).is_some() {
                return Ok(());
            }
        }

        let response = self.http_client.get(url).send().await?;
        let mut binfo = manifest::BlobInfo::new_from_ingest(Box::new(response), &mut storage).await?;

        binfo.set_url(url);
        self.manifest.insert_or_update(file_name, binfo, self.nbe);
        self.manifest_modified = true;

        Ok(())
    }

    /// Rewrite the manifest if needed.
    pub fn rewrite_manifest(&mut self) -> Result<()> {
        if !self.manifest_modified {
            return Ok(());
        }

        let path = self
            .manifest_path
            .as_ref()
            .map(|pb| pb.as_ref())
            .unwrap_or_else(|| Path::new(manifest::MANIFEST_STEM));
        let text = toml::ser::to_string_pretty(&self.manifest)?;
        let mut f = File::create(&path)?;
        ctry!(write!(f, "{}", text); "couldn\'t write manifest file {}", path.display());
        self.manifest_modified = false;
        Ok(())
    }

    /// Provide a blob in the current directory.
    ///
    /// We should eventually have some method to identify which of several
    /// Storage backends has the blob we want, but for now there's just one.
    pub fn provide_blob(&mut self, name: &str) -> Result<()> {
        let storage = ctry!(self.get_storage(); "cannot open storage backend");

        let storage_path = {
            let binfo = match self.manifest.lookup(name) {
                Some(b) => b,
                None => {
                    return err_msg!("no known blob named \"{}\"", name);
                }
            };

            match storage.get_path(binfo.digest())? {
                Some(p) => p,
                None => {
                    return err_msg!("blob \"{}\" not available as standalone file", name);
                }
            }
        };

        let dest_path = Path::new(name);

        ctry!(io::try_remove_file(&dest_path);
              "couldn\'t remove existing file {}", dest_path.display());
        ctry!(fs::hard_link(&storage_path, &dest_path);
              "couldn\'t link {} to {}", storage_path.display(), dest_path.display());

        Ok(())
    }

    /// Get a Read stream to the named blob.
    pub fn open_blob(&mut self, name: &str) -> Result<Box<dyn Read>> {
        let storage = ctry!(self.get_storage(); "cannot open storage backend");

        let binfo = match self.manifest.lookup(name) {
            Some(b) => b,
            None => {
                return err_msg!("no known blob named \"{}\"", name);
            }
        };

        match storage.open(binfo.digest())? {
            Some(r) => Ok(r),
            None => {
                return err_msg!("blob \"{}\" not available", name);
            }
        }
    }
}
