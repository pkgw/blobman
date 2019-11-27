// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Storing blobs on the filesystem.

*/

use async_trait::async_trait;
use mkstemp::TempFile;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::{AsyncChunks, Storage};
use crate::{
    ctry,
    digest::{Digest, DigestComputer, DigestData},
    err_msg,
    errors::Result,
    io,
};

/// A storage backend that arranges files on the filesystem.
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

#[async_trait]
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

    async fn ingest(
        &mut self,
        mut source: Box<dyn AsyncChunks + Send>,
    ) -> Result<(u64, DigestData)> {
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

        // Stream into the tempfile.

        let mut computer: DigestComputer = Default::default();
        let mut total_size = 0;

        let temp_path = {
            let mut tempfile = ctry!(
                TempFile::new(template, false);
                "couldn\'t create temporary file with template {}", template
            );
            let temp_path = tempfile.path().to_owned();

            // TODO: clean up from errors!!!

            while let Some(data) = source.get_chunk().await? {
                total_size += data.len() as u64;
                computer.input(&data);
                tempfile.write_all(&data)?;
            }

            temp_path
        };

        // Finalize

        let digest: DigestData = computer.into();
        let dest_path = ctry!(
            digest.create_two_part_path(&self.prefix);
            "couldn't make directories in {}", self.prefix.display()
        );
        ctry!(
            fs::rename(&temp_path, &dest_path);
            "couldn't rename {} to {}", temp_path, dest_path.display()
        );

        let mut perms =
            ctry!(fs::metadata(&dest_path); "couldn't get info for file {}", dest_path.display())
                .permissions();
        perms.set_readonly(true);
        ctry!(fs::set_permissions(&dest_path, perms); "couldn\'t make file {} read-only", dest_path.display());

        Ok((total_size, digest))
    }
}
