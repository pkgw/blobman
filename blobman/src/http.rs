// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Retrieving blobs over HTTP(S).

This code is very directly derived from the `examples/hyper-client.rs` file
provided in the `tokio-tls` Git repository.

*/

use reqwest;
use std::io;
use std::str;
use tokio::runtime::Runtime;

use crate::errors::Result;

/// Download over HTTP or HTTPS into a Write object.
///
/// Because our HTTP layer is fancy and asynchronous while the rest of our
/// operation is synchronous, we can't just return a simple Read stream.
pub fn download<W: io::Write>(
    client: &mut reqwest::Client,
    uri: &str,
    dest: W,
) -> Result<u64> {
    let rt = Runtime::new()?;
    rt.block_on(download_async(client, uri, dest))
}


async fn download_async<W: io::Write>(
    client: &mut reqwest::Client,
    uri: &str,
    mut dest: W,
) -> Result<u64> {
    let mut resp = client.get(uri).send().await?;
    let mut n_bytes = 0;

    while let Some(chunk) = resp.chunk().await? {
        n_bytes += chunk.len() as u64;
        dest.write_all(&chunk)?;
    }

    Ok(n_bytes)
}
