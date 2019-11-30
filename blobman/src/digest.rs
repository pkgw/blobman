// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! Helpers to tidy up hangling of SHA256 digests.
//!
//! This module is ripped off from the `errors` module used by the
//! [Tectonic](https://github.com/tectonic-typesetting/tectonic) typesetting
//! engine. (Which the author of this module also wrote.)

use serde;
pub use sha2::Digest;
pub use sha2::Sha256 as DigestComputer;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::str::FromStr;
use std::string::ToString;

use crate::errors::{Error, ErrorKind, Result};

/// Return *bytes* as represented in a hexadecimal string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .concat()
}

/// Decode a hexadecimal string into a byte vector.
pub fn hex_to_bytes(text: &str, dest: &mut [u8]) -> Result<()> {
    let n = dest.len();
    let text_len = text.len();

    if text_len != 2 * n {
        return Err(ErrorKind::BadLength(2 * n, text_len).into());
    }

    for i in 0..n {
        dest[i] = u8::from_str_radix(&text[i * 2..(i + 1) * 2], 16)?;
    }

    Ok(())
}

// The specific implementation we're using: SHA256.

const N_BYTES: usize = 32;

/// Create an object that can compute a digest from a byte stream.
pub fn create() -> DigestComputer {
    Default::default()
}

/// A vector of bytes holding a cryptographic digest.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestData([u8; N_BYTES]);

impl DigestData {
    /// Return a DigestData that is all zeros.
    pub fn zeros() -> DigestData {
        DigestData([0u8; N_BYTES])
    }

    /// Return the digest of a zero-length byte stream.
    ///
    /// Note that this is not necessarily all zeros.
    pub fn of_nothing() -> DigestData {
        let dc = create();
        Self::from(dc)
    }

    /// Given a base path, create a child path from this digest's value. The
    /// child path has a subdirectory from the hex value of the first byte of
    /// the digest, then a name consisting of the rest of the hex data. **The
    /// first-byte subdirectory and all parent directories are created when
    /// you call this function!**
    pub fn create_two_part_path(&self, base: &Path) -> Result<PathBuf> {
        let mut p = base.to_path_buf();
        p.push(format!("{:02x}", self.0[0]));
        fs::create_dir_all(&p)?;
        p.push(bytes_to_hex(&self.0[1..]));
        Ok(p)
    }
}

impl ToString for DigestData {
    fn to_string(&self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl FromStr for DigestData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut result = DigestData::zeros();
        hex_to_bytes(s, &mut result.0)?;
        Ok(result)
    }
}

impl<'de> serde::Deserialize<'de> for DigestData {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DigestDataVisitor;

        impl<'de> serde::de::Visitor<'de> for DigestDataVisitor {
            type Value = DigestData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hexadecimal string")
            }

            fn visit_str<E>(self, v: &str) -> StdResult<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut result = DigestData::zeros();
                match hex_to_bytes(v, &mut result.0) {
                    Ok(_) => Ok(result),
                    Err(e) => Err(E::custom(e.description())),
                }
            }
        }

        deserializer.deserialize_str(DigestDataVisitor)
    }
}

impl serde::Serialize for DigestData {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<DigestComputer> for DigestData {
    fn from(s: DigestComputer) -> DigestData {
        let mut result = DigestData::zeros();
        let res = s.result();
        result.0.copy_from_slice(res.as_slice());
        result
    }
}
