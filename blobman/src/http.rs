// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
Retrieving blobs over HTTP(S).

This code is very directly derived from the `examples/hyper-client.rs` file
provided in the `tokio-tls` Git repository.

*/

use std::io;
use std::str;

use crate::err_msg;
use crate::errors::Result;

/// Download over HTTP or HTTPS into a Write object.
///
/// Because our HTTP layer is fancy and asynchronous while the rest of our
/// operation is synchronous, we can't just return a simple Read stream.
pub fn download<W: io::Write>(_uri: &str, mut _dest: W) -> Result<u64> {
    err_msg!("not implemented")
    // let mut core = Core::new()?;
    //
    // // Create a custom "connector" for Hyper which will route connections
    // // through the `TlsConnector` we create here after routing them through
    // // `HttpConnector` first.
    // let tls_cx = TlsConnector::builder().build()?;
    // let mut connector = HttpsConnector {
    //     tls: Arc::new(tls_cx),
    //     http: HttpConnector::new(2, &core.handle()),
    // };
    // connector.http.enforce_http(false);
    // let client = Client::configure()
    //     .connector(connector)
    //     .build(&core.handle());
    //
    // // Send off a request. This will just fetch the headers; the body won't be
    // // downloaded yet.
    //
    // const MAX_REDIRECTS: usize = 16;
    // let mut parsed: Uri = uri.parse()?;
    // let mut req = Request::new(Method::Get, parsed.clone());
    // let mut response;
    // let mut attempt_num: usize = 0;
    //
    // loop {
    //     // liveness checker doesn't like `for attempt_num in 0..MAX_REDIRECTS`
    //     attempt_num += 1;
    //     if attempt_num > MAX_REDIRECTS {
    //         return err_msg!("failed to download {}: too many redirection", uri);
    //     }
    //
    //     response = core.run(client.request(req))?;
    //     let status = response.status();
    //
    //     if status.is_success() {
    //         break;
    //     }
    //
    //     if status.is_redirection() {
    //         let loc_hdr = match response.headers().get::<Location>() {
    //             Some(h) => h,
    //             None => {
    //                 return err_msg!("illegal redirect from {}: no Location header", parsed);
    //             }
    //         };
    //         parsed = loc_hdr.parse()?;
    //         req = Request::new(Method::Get, parsed.clone());
    //         continue;
    //     }
    //
    //     return err_msg!(
    //         "failed to download {}: got non-successful HTTP status {}",
    //         uri,
    //         response.status()
    //     );
    // }
    //
    // // Finish off our request by fetching the body.
    //
    // let mut stream = response.body();
    // let mut n_bytes = 0;
    //
    // loop {
    //     stream = match core.run(stream.into_future()) {
    //         Err((e, _)) => {
    //             return Err(e.into());
    //         }
    //         Ok((chunk, next)) => {
    //             match chunk {
    //                 None => {
    //                     break;
    //                 }
    //                 Some(c) => {
    //                     n_bytes += c.len() as u64;
    //                     dest.write_all(&c)?;
    //                 }
    //             };
    //             next
    //         }
    //     };
    // }
    //
    // Ok(n_bytes)
}
