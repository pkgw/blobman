// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
I/O helpers.
*/

use std::fs::{self, File};
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;

use crate::errors::Result;

/// Try to open a file, returning `Ok(None)` if it does not exist.
///
/// This is a simple wrapper around an operation that I’ve found to be
/// relatively common: often there are cases where you want to try to open a
/// file, but it is not an error if it doesn’t exist.
pub fn try_open<P: AsRef<Path>>(path: P) -> Result<Option<File>> {
    match File::open(path) {
        Ok(f) => Ok(Some(f)),
        Err(e) => {
            if e.kind() == IoErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e.into())
            }
        }
    }
}

/// Try to remove a file, ignoring the failure if it does not exist.
pub fn try_remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == IoErrorKind::NotFound {
                Ok(())
            } else {
                Err(e.into())
            }
        }
    }
}
