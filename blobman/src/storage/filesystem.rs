// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Storing blobs on the filesystem.

*/

use mkstemp::TempFile;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::{StagingCookie, Storage};
use crate::digest::DigestData;
use crate::errors::Result;
use crate::io;
use crate::{ctry, err_msg};

/// A storage backend that arranges files on the filesystem.
#[derive(Debug)]
pub struct FilesystemStorage {
    prefix: PathBuf,
    next_staging_cookie: usize,
    staging_paths: HashMap<usize, PathBuf>,
}

impl FilesystemStorage {
    /// Create and return a new FilesystemStorage object.
    pub fn new<P: AsRef<OsStr>>(prefix: &P) -> Self {
        Self {
            prefix: PathBuf::from(prefix),
            next_staging_cookie: 0,
            staging_paths: HashMap::new(),
        }
    }
}

impl Storage for FilesystemStorage {
    fn get_path(&self, digest: &DigestData) -> Result<Option<PathBuf>> {
        let path = ctry!(digest.create_two_part_path(&self.prefix);
                         "couldn't make directories in {}", self.prefix.display());

        if path.exists() {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    fn open(&self, digest: &DigestData) -> Result<Option<Box<dyn Read>>> {
        let path = ctry!(digest.create_two_part_path(&self.prefix);
                         "couldn't make directories in {}", self.prefix.display());
        Ok(io::try_open(path)?.map(|f| Box::new(f) as Box<dyn Read>))
    }

    fn start_staging<'a>(&'a mut self) -> Result<(Box<dyn Write>, StagingCookie)> {
        let mut p = self.prefix.clone();
        p.push("staging.XXXXXXXX");

        // Unfortunately mkstemp-rs wants its input paths to be str's, not
        // OsStr's. To be paranoid we refuse to run if we can't convert
        // successfully.

        let template = match p.to_str() {
            Some(t) => t,
            None => {
                return err_msg!(
                    "cannot save data to destination {}: path is not Unicode-compatible",
                    p.display()
                );
            }
        };

        let tempfile = ctry!(TempFile::new(template, false); "couldn\'t create temporary file with template {}", template);

        let cookie = self.next_staging_cookie;
        self.next_staging_cookie += 1;
        self.staging_paths
            .insert(cookie, PathBuf::from(tempfile.path()));

        Ok((Box::new(tempfile), cookie))
    }

    fn finish_staging(&mut self, cookie: StagingCookie, digest: &DigestData) -> Result<()> {
        let src_path = self.staging_paths.remove(&cookie).unwrap();
        let dest_path = ctry!(digest.create_two_part_path(&self.prefix);
                              "couldn't make directories in {}", self.prefix.display());
        ctry!(fs::rename(&src_path, &dest_path);
              "couldn't rename {} to {}", src_path.display(), dest_path.display());

        let mut perms =
            ctry!(fs::metadata(&dest_path); "couldn't get info for file {}", dest_path.display())
                .permissions();
        perms.set_readonly(true);
        ctry!(fs::set_permissions(&dest_path, perms); "couldn\'t make file {} read-only", dest_path.display());

        Ok(())
    }
}
