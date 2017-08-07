// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Retrieving blobs over HTTP(S).

This code is very directly derived from the `examples/hyper-client.rs` file
provided in the `tokio-tls` Git repository.

*/

use futures::future::{err, Future};
use futures::stream::Stream;
use hyper::client::HttpConnector;
use hyper::{Client, Request, Method, Uri};
use native_tls::TlsConnector;
use std::io;
use std::str;
use std::sync::Arc;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_service::Service;
use tokio_tls::{TlsConnectorExt, TlsStream};

use errors::Result;


struct HttpsConnector {
    tls: Arc<TlsConnector>,
    http: HttpConnector,
}


impl Service for HttpsConnector {
    type Request = Uri;
    type Response = TlsStream<TcpStream>;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = io::Error>>;

    fn call(&self, uri: Uri) -> Self::Future {
        // Right now this is intended to showcase `https`, but you could also
        // adapt this to return something like `MaybeTls<T>` where some
        // clients resolve to TLS streams (https) and others resolve to normal
        // TCP streams (http)
        if uri.scheme() != Some("https") {
            return err(io::Error::new(io::ErrorKind::Other,
                                      "only works with https")).boxed()
        }

        // Look up the host that we're connecting to as we're going to validate
        // this as part of the TLS handshake.
        let host = match uri.host() {
            Some(s) => s.to_string(),
            None =>  {
                return err(io::Error::new(io::ErrorKind::Other,
                                          "missing host")).boxed()
            }
        };

        // Delegate to the standard `HttpConnector` type to create a connected
        // TCP socket. Once we've got that socket initiate the TLS handshake
        // with the host name that's provided in the URI we extracted above.
        let tls_cx = self.tls.clone();
        Box::new(self.http.call(uri).and_then(move |tcp| {
            tls_cx.connect_async(&host, tcp)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }))
    }
}



/// Download over HTTPS into a Write object.
///
/// Because our HTTP layer is fancy and asynchronous while the rest of our
/// operation is synchronous, we can't just return a simple Read stream.
pub fn download<W: io::Write>(uri: &str, mut dest: W) -> Result<u64> {
    let mut core = Core::new()?;

    // Create a custom "connector" for Hyper which will route connections
    // through the `TlsConnector` we create here after routing them through
    // `HttpConnector` first.
    let tls_cx = TlsConnector::builder()?.build()?;
    let mut connector = HttpsConnector {
        tls: Arc::new(tls_cx),
        http: HttpConnector::new(2, &core.handle()),
    };
    connector.http.enforce_http(false);
    let client = Client::configure()
                    .connector(connector)
                    .build(&core.handle());

    // Send off a request. This will just fetch the headers; the body won't be
    // downloaded yet.

    let parsed = uri.parse()?;
    let req = Request::new(Method::Get, parsed);
    let response = core.run(client.request(req))?;

    if !response.status().is_success() {
        return err_msg!("failed to download {}: got non-successful HTTP status {}",
                        uri, response.status());
    }

    // Finish off our request by fetching the body.

    let mut stream = response.body();
    let mut n_bytes = 0;

    loop {
        stream = match core.run(stream.into_future()) {
            Err((e, _)) => {
                return Err(e.into());
            },
            Ok((chunk, next)) => {
                match chunk {
                    None => { break; },
                    Some(c) => {
                        n_bytes += c.len() as u64;
                        dest.write_all(&c)?;
                    },
                };
                next
            }
        };
    };

    Ok(n_bytes)
}
