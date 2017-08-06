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
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate sha2;
extern crate termcolor;
extern crate toml;

#[macro_use] pub mod notify; // must come first to provide macros for other modules
#[macro_use] pub mod errors;
pub mod config;
pub mod digest;
pub mod io;
pub mod manifest;


use std::path::PathBuf;

use errors::Result;


const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo {name: "blobman", author: "BlobmanProject"};


/// A session in which we do stuff.
pub struct Session<'a, B: 'a + notify::NotificationBackend> {
    config: &'a config::UserConfig,
    nbe: &'a mut B,
    manifest_path: Option<PathBuf>,
    manifest: manifest::Manifest,
}


impl<'a, B: notify::NotificationBackend> Session<'a, B> {
    /// Create and return a new Session.
    pub fn new(config: &'a config::UserConfig, nbe: &'a mut B) -> Result<Self> {
        let (manifest, manifest_path) = manifest::Manifest::find()?;

        Ok(Self {
            config: config,
            nbe: nbe,
            manifest_path: manifest_path,
            manifest: manifest,
        })
    }

    /// Fetch a blob from a URL and ingest it.
    pub fn fetch_url(&mut self, url: &str) -> Result<()> {
        bm_note!(self.nbe, "should fetch: {}", url);
        Ok(())
    }
}
