// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

#[macro_use] extern crate blobman;
extern crate clap;

use clap::{Arg, ArgMatches, App};
use std::process;

use blobman::errors::Result;
use blobman::notify::{ChatterLevel, NotificationBackend};
use blobman::notify::termcolor::TermcolorNotificationBackend;


fn inner(_matches: ArgMatches, nbe: &mut TermcolorNotificationBackend) -> Result<i32> {
    bm_note!(nbe, "Here we are.");
    Ok(0)
}


fn main() {
    let matches = App::new("blobman")
        .version("0.1.0")
        .about("Manage data files.")
        .arg(Arg::with_name("chatter_level")
             .long("chatter")
             .short("c")
             .value_name("LEVEL")
             .help("How much chatter to print when running.")
             .possible_values(&["default", "minimal"])
             .default_value("default"))
        .get_matches ();

    let chatter = match matches.value_of("chatter_level").unwrap() {
        "default" => ChatterLevel::Normal,
        "minimal" => ChatterLevel::Minimal,
        _ => unreachable!()
    };

    // Set up colorized output. This comes after the config because you could
    // imagine wanting to be able to configure the colorization (which is
    // something I'd be relatively OK with since it'd only affect the progam
    // UI, not the processing results).

    let mut nbe = TermcolorNotificationBackend::new(chatter);

    // Now that we've got colorized output, we're to pass off to the inner
    // function ... all so that we can print out the word "error:" in red.
    // This code parallels various bits of the `error_chain` crate.

    process::exit(match inner(matches, &mut nbe) {
        Ok(ret) => ret,

        Err(ref e) => {
            nbe.bare_error(e);
            1
        }
    })
}
