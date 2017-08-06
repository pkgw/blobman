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
extern crate hyper;
extern crate hyper_native_tls;
extern crate mkstemp;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate sha2;
extern crate termcolor;
extern crate toml;

#[macro_use] pub mod notify; // must come first to provide macros for other modules
#[macro_use] pub mod errors;
pub mod config;
pub mod digest;
pub mod http;
pub mod io;
pub mod manifest;
pub mod storage;


use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use errors::Result;


const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo {name: "blobman", author: "BlobmanProject"};


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
    pub fn fetch_url(&mut self, url: &str) -> Result<()> {
        let parsed = hyper::Url::parse(url)?;
        let file_name = match parsed.path_segments() {
            None => { return err_msg!("cannot extract a filename from the URL {}", url); },
            Some(segments) => match segments.last() {
                None => { return err_msg!("cannot extract a filename from the URL {}", url); },
                Some(s) => s,
            },
        };

        let mut storage = ctry!(self.get_storage(); "cannot open storage backend");
        let mut binfo = {
            let source = http::download(url)?;
            manifest::BlobInfo::new_from_ingest(source, &mut *storage)?
        };
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
        bm_note!(self.nbe, "wrote manifest {}", path.display());
        self.manifest_modified = false;
        Ok(())
    }
}
