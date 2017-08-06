// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Retrieving blobs over HTTP(S).

We're currently fixed to the Hyper version 0.10, which is synchronous. It
seems overly difficult to get asynchronous downloading going just yet without
making the whole application synchronous.
*/

use hyper::{self, Client, Url};
use hyper::client::{Response, RedirectPolicy};
use hyper::header::{Headers, Range};
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;
use hyper_native_tls::NativeTlsClient;

use errors::{Error, Result, ResultExt};


/// Open a stream to download a file.
pub fn download(url: &str) -> Result<Response> {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let mut client = Client::with_connector(connector);
    let req = client.get(url);
    let res = ctry!(req.send(); "couldn\'t request Web address {}", url);

    if !res.status.is_success() {
        return Err(Error::from(hyper::Error::Status)).chain_err(|| format!("couldn\'t fetch {}", url));
    }

    Ok(res)
}
