// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

#![recursion_limit = "1024"] // "error_chain can recurse deeply"

extern crate app_dirs;
#[macro_use] extern crate error_chain;
extern crate sha2;
extern crate toml;

pub mod errors;
