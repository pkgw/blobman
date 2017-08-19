// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
This crate provides a framework for managing binary “blobs” of data.

A blob is just a file whose contents the framework does not care about, except
that blobman’s job is to ensure that the contents are exactly what the caller
expects.

*/

#![recursion_limit = "1024"] // "error_chain can recurse deeply"
#![deny(missing_docs)]

extern crate app_dirs;
#[macro_use] extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate mkstemp;
extern crate native_tls;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate sha2;
extern crate termcolor;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_tls;
extern crate toml;

#[macro_use] pub mod notify; // must come first to provide macros for other modules
#[macro_use] pub mod errors;
pub mod config;
pub mod digest;
pub mod http;
pub mod io;
pub mod manifest;
pub mod storage;


use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use errors::{Error, Result};


const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo {name: "blobman", author: "BlobmanProject"};


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
        static S: &'static[&str] = &["update", "trust"];
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
pub struct Session<'a, B: 'a + notify::NotificationBackend> {
    config: &'a config::UserConfig,
    nbe: &'a mut B,
    manifest: manifest::Manifest,
    manifest_path: Option<PathBuf>,
    manifest_modified: bool,
}


impl<'a, B: notify::NotificationBackend> Session<'a, B> {
    /// Create and return a new Session.
    pub fn new(config: &'a config::UserConfig, nbe: &'a mut B) -> Result<Self> {
        let (manifest, manifest_path) = manifest::Manifest::find()?;

        Ok(Self {
            config: config,
            nbe: nbe,
            manifest: manifest,
            manifest_path: manifest_path,
            manifest_modified: false,
        })
    }


    /// Get a storage backend for this session.
    ///
    /// TODO: Maybe we'll one day have multiple backends and some fancy way to
    /// decide which one to use. For now we just use one. We do, however, use
    /// a trait object since we envision have runtime-configurable backends
    /// here.
    pub fn get_storage(&mut self) -> Result<Box<storage::Storage>> {
        self.config.get_storage(self.nbe)
    }

    /// Fetch a blob from a URL and ingest it.
    pub fn ingest_from_url(&mut self, mode: IngestMode, url: &str, name: Option<&str>) -> Result<()> {
        let parsed: hyper::Uri = url.parse()?;
        let file_name = match name {
            Some(n) => n,
            None => match parsed.path().split("/").last() {
                None => { return err_msg!("cannot extract a filename from the URL {}", url); },
                Some(s) => s,
            },
        };

        let mut storage = ctry!(self.get_storage(); "cannot open storage backend");

        if let IngestMode::TrustExisting = mode {
            if self.manifest.lookup(file_name).is_some() {
                return Ok(());
            }
        }

        let mut binfo = manifest::BlobInfo::new_from_ingest(|w| http::download(url, w), &mut *storage)?;
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

        let path = self.manifest_path
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
                None => { return err_msg!("no known blob named \"{}\"", name); },
            };

            match storage.get_path(binfo.digest())? {
                Some(p) => p,
                None => { return err_msg!("blob \"{}\" not available as standalone file", name); },
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
    pub fn open_blob(&mut self, name: &str) -> Result<Box<Read>> {
        let storage = ctry!(self.get_storage(); "cannot open storage backend");

        let binfo = match self.manifest.lookup(name) {
            Some(b) => b,
            None => { return err_msg!("no known blob named \"{}\"", name); },
        };

        match storage.open(binfo.digest())? {
            Some(r) => Ok(r),
            None => { return err_msg!("blob \"{}\" not available", name); },
        }
    }
}
