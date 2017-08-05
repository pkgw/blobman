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
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate sha2;
extern crate termcolor;
extern crate toml;

#[macro_use] pub mod notify; // must come first to provide macros for other modules
#[macro_use] pub mod errors;
pub mod config;
pub mod io;

const APP_INFO: app_dirs::AppInfo = app_dirs::AppInfo {name: "blobman", author: "BlobmanProject"};
