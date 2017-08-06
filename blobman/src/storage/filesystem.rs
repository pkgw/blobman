// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Storing blobs on the filesystem.

*/

use mkstemp::TempFile;
use std::ffi::OsStr;
use std::fs;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::{Path, PathBuf};

use digest::DigestData;
use errors::Result;
use super::{StagerOps, Storage};


/// A storage backend that arranges files on the filesystem
#[derive(Debug)]
pub struct FilesystemStorage {
    prefix: PathBuf,
}


impl FilesystemStorage {
    /// Create and return a new FilesystemStorage object.
    pub fn new<P: AsRef<OsStr>>(prefix: &P) -> Self {
        Self {
            prefix: PathBuf::from(prefix),
        }
    }
}


impl<'a> Storage<'a> for FilesystemStorage {
    type Stager = FilesystemStager<'a>;

    fn new_stager(&'a mut self) -> Result<Self::Stager> {
        let mut p = self.prefix.clone();
        p.push("staging.XXXXXXXX");

        // Unfortunately mkstemp-rs wants its input paths to be str's, not
        // OsStr's. To be paranoid we refuse to run if we can't convert
        // successfully.

        let template = match p.to_str() {
            Some(t) => t,
            None => {
                return err_msg!("cannot save data to destination {}: path is not Unicode-compatible", p.display());
            }
        };

        let tempfile = ctry!(TempFile::new(template, false); "couldn\'t create temporary file with template {}", template);

        Ok(Self::Stager {
            storage: self,
            tempfile: tempfile,
        })
    }
}


/// A type for staging blobs onto a FilesystemStorage instance.
// Can't be Debug because TempFile isn't.
pub struct FilesystemStager<'a> {
    storage: &'a mut FilesystemStorage,
    tempfile: TempFile,
}


impl<'a> Write for FilesystemStager<'a> {
    fn write(&mut self, data: &[u8]) -> IoResult<usize> {
        self.tempfile.write(data)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.tempfile.flush()
    }
}


impl<'a> StagerOps for FilesystemStager<'a> {
    /// Rename the temporary file to its final destination and make it
    /// read-only.
    fn finish(mut self, digest: &DigestData) -> Result<()> {
        let src_path = self.tempfile.path();
        let dest_path = ctry!(digest.create_two_part_path(&self.storage.prefix);
                              "couldn't make directories in {}", self.storage.prefix.display());
        ctry!(fs::rename(src_path, &dest_path);
              "couldn't rename {} to {}", src_path, dest_path.display());

        let mut perms = ctry!(fs::metadata(&dest_path); "couldn't get info for file {}", dest_path.display()).permissions();
        perms.set_readonly(true);
        ctry!(fs::set_permissions(&dest_path, perms); "couldn\'t make file {} read-only", dest_path.display());

        Ok(())
    }
}
