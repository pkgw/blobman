// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Configuration of the blobman framework.

use app_dirs::{app_dir, app_root, AppDataType};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use toml;

use crate::errors::Result;
use crate::io;
use crate::notify::NotificationBackend;
use crate::storage::{filesystem, Storage};
use crate::{bm_note, bm_warning, err_msg};

const DEFAULT_CONFIG: &'static str = r#"[[storage]]
location = {type = "user_cache", loc = "blobman"}
"#;

/// Deserialized user-level configuration information.
#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    storage: Vec<StorageInfo>,
}

/// Information about a storage area that blobman can use.
#[derive(Debug, Deserialize, Serialize)]
pub struct StorageInfo {
    location: StorageLocation,
}

/// A location where blobs can be stored.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "loc")]
pub enum StorageLocation {
    /// An absolute path on the filesystem.
    #[serde(rename = "filesystem")]
    Filesystem(PathBuf),

    /// A filesystem path relative to the user’s personal cache directory.
    ///
    /// This is processed through app_dirs and therefore must be a String,
    /// not a full PathBuf.
    #[serde(rename = "user_cache")]
    UserCache(String),
}

impl StorageInfo {
    /// Open a storage backend based on the configured information.
    ///
    /// Because the StorageInfo multiplexes over different backend
    /// implementations, we return a trait object.
    pub fn open(&self) -> Result<Box<dyn Storage>> {
        match self.location {
            StorageLocation::Filesystem(ref prefix) => {
                if !prefix.is_absolute() {
                    return err_msg!(
                        "the path associated with filesystem storage must be absolute; got {}",
                        prefix.display()
                    );
                }
                Ok(Box::new(filesystem::FilesystemStorage::new(prefix)))
            }
            StorageLocation::UserCache(ref subdir) => {
                let d = app_dir(AppDataType::UserCache, &crate::APP_INFO, subdir)?;
                Ok(Box::new(filesystem::FilesystemStorage::new(&d)))
            }
        }
    }
}

impl UserConfig {
    /// Read the user-level configuration data.
    pub fn open<B: NotificationBackend>(nbe: &mut B) -> Result<UserConfig> {
        let mut cfg_path = app_root(AppDataType::UserConfig, &crate::APP_INFO)?;
        cfg_path.push("config.toml");

        let config = match io::try_open(&cfg_path)? {
            Some(mut f) => {
                let mut buf = Vec::<u8>::new();
                f.read_to_end(&mut buf)?;
                toml::from_slice(&buf)?
            }
            None => {
                let mut f = File::create(&cfg_path)?;
                write!(f, "{}", DEFAULT_CONFIG)?;
                bm_note!(nbe, "created configuration file {}", cfg_path.display());
                toml::from_str(DEFAULT_CONFIG)?
            }
        };

        Ok(config)
    }

    /// Get a storage backend.
    ///
    /// This is a bit of a hack; the main logic should probably be confined to
    /// the Session type.
    pub fn get_storage<B: NotificationBackend>(&self, nbe: &mut B) -> Result<Box<dyn Storage>> {
        if self.storage.len() == 0 {
            return err_msg!("no storage backends defined in the config file");
        }

        if self.storage.len() > 1 {
            bm_warning!(
                nbe,
                "I only pay attention to the first storage area that's been configured"
            );
        }

        self.storage[0].open()
    }
}
