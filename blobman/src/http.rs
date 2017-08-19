// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Retrieving blobs over HTTP(S).

This code is very directly derived from the `examples/hyper-client.rs` file
provided in the `tokio-tls` Git repository.

*/

use bytes::buf::{Buf, BufMut};
use futures::Poll;
use futures::future::{err, Future};
use futures::stream::Stream;
use hyper::{Client, Request, Method, Uri};
use hyper::client::HttpConnector;
use hyper::header::Location;
use native_tls::TlsConnector;
use std::io::{self, Read, Write};
use std::str;
use std::sync::Arc;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder, Framed, FramedParts};
use tokio_service::Service;
use tokio_tls::{TlsConnectorExt, TlsStream};

use errors::Result;


#[derive(Debug)]
enum MaybeTls<T> {
    Yes(TlsStream<T>),
    No(T),
}

impl<T: Read + Write> Read for MaybeTls<T> {
    // TODO: implement rest of functions in case sub-types have specialized
    // implementations
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            MaybeTls::Yes(ref mut s) => s.read(buf),
            MaybeTls::No(ref mut s) => s.read(buf),
        }
    }
}

impl<T: Read + Write> Write for MaybeTls<T> {
    // TODO: implement rest of functions in case sub-types have specialized
    // implementations
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            MaybeTls::Yes(ref mut s) => s.write(buf),
            MaybeTls::No(ref mut s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            MaybeTls::Yes(ref mut s) => s.flush(),
            MaybeTls::No(ref mut s) => s.flush(),
        }
    }
}

impl<T: AsyncRead + AsyncWrite> AsyncRead for MaybeTls<T> {
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        match *self {
            MaybeTls::Yes(ref s) => s.prepare_uninitialized_buffer(buf),
            MaybeTls::No(ref s) => s.prepare_uninitialized_buffer(buf),
        }
    }

    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> where Self: Sized {
        match *self {
            MaybeTls::Yes(ref mut s) => s.read_buf(buf),
            MaybeTls::No(ref mut s) => s.read_buf(buf),
        }
    }

    fn framed<C: Encoder + Decoder>(self, codec: C) -> Framed<Self, C> where Self: AsyncWrite + Sized {
        match self {
            MaybeTls::Yes(s) => {
                let (parts, codec) = s.framed(codec).into_parts_and_codec();
                Framed::from_parts(FramedParts {
                    inner: MaybeTls::Yes(parts.inner),
                    readbuf: parts.readbuf,
                    writebuf: parts.writebuf,
                }, codec)
            },
            MaybeTls::No(s) => {
                let (parts, codec) = s.framed(codec).into_parts_and_codec();
                Framed::from_parts(FramedParts {
                    inner: MaybeTls::No(parts.inner),
                    readbuf: parts.readbuf,
                    writebuf: parts.writebuf,
                }, codec)
            },
        }
    }

    //fn split(self) -> (ReadHalf<Self>, WriteHalf<Self>) where Self: AsyncWrite + Sized {}
}

impl<T: AsyncWrite + AsyncRead> AsyncWrite for MaybeTls<T> {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        match *self {
            MaybeTls::Yes(ref mut s) => s.shutdown(),
            MaybeTls::No(ref mut s) => s.shutdown(),
        }
    }

    fn write_buf<B: Buf>(&mut self, buf: &mut B) -> Poll<usize, io::Error> where Self: Sized {
        match *self {
            MaybeTls::Yes(ref mut s) => s.write_buf(buf),
            MaybeTls::No(ref mut s) => s.write_buf(buf),
        }
    }
}


struct HttpsConnector {
    tls: Arc<TlsConnector>,
    http: HttpConnector,
}

impl Service for HttpsConnector {
    type Request = Uri;
    type Response = MaybeTls<TcpStream>;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, uri: Uri) -> Self::Future {
        // The simple case:
        if uri.scheme() == Some("http") {
            return Box::new(self.http.call(uri).map(|tcp| MaybeTls::No(tcp)));
        }

        // We only support one other option at the moment ...
        if uri.scheme() != Some("https") {
            return err(io::Error::new(io::ErrorKind::Other,
                                      "only HTTP and HTTPS are supported")).boxed()
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
            tls_cx
                .connect_async(&host, tcp)
                .map(|tls| MaybeTls::Yes(tls))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }))
    }
}


/// Download over HTTP or HTTPS into a Write object.
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

    const MAX_REDIRECTS: usize = 16;
    let mut parsed: Uri = uri.parse()?;
    let mut req = Request::new(Method::Get, parsed.clone());
    let mut response;
    let mut attempt_num: usize = 0;

    loop { // liveness checker doesn't like `for attempt_num in 0..MAX_REDIRECTS`
        attempt_num += 1;
        if attempt_num > MAX_REDIRECTS {
            return err_msg!("failed to download {}: too many redirection", uri);
        }

        response = core.run(client.request(req))?;
        let status = response.status();

        if status.is_success() {
            break;
        }

        if status.is_redirection() {
            let loc_hdr = match response.headers().get::<Location>() {
                Some(h) => h,
                None => {
                    return err_msg!("illegal redirect from {}: no Location header", parsed);
                },
            };
            parsed = loc_hdr.parse()?;
            req = Request::new(Method::Get, parsed.clone());
            continue;
        }

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
