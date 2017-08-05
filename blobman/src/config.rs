// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

//! Configuration of the blobman framework.

use std::io::{Read, Write};
use std::fs::File;
use std::path::PathBuf;

use app_dirs::{app_root, AppDataType};
use toml;

use errors::Result;
use io;
use notify::NotificationBackend;


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
    #[serde(rename = "user_cache")]
    UserCache(PathBuf),
}


impl UserConfig {
    /// Read the user-level configuration data.
    pub fn open<B: NotificationBackend>(nbe: &mut B) -> Result<UserConfig> {
        let mut cfg_path = app_root(AppDataType::UserConfig, &::APP_INFO)?;
        cfg_path.push("config.toml");

        let config = match io::try_open(&cfg_path)? {
            Some(mut f) => {
                let mut buf = Vec::<u8>::new();
                f.read_to_end(&mut buf)?;
                toml::from_slice(&buf)?
            },
            None => {
                let mut f = File::create(&cfg_path)?;
                write!(f, "{}", DEFAULT_CONFIG)?;
                bm_note!(nbe, "created configuration file {}", cfg_path.display());
                toml::from_str(DEFAULT_CONFIG)?
            },
        };

        Ok(config)
    }
}
