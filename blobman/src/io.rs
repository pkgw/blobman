// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
I/O helpers.
*/

use std::fs::File;
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;

use errors::Result;


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
