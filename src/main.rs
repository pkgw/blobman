// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

#[macro_use] extern crate blobman;
extern crate clap;

use clap::{Arg, ArgMatches, App};
use std::process;

use blobman::config::UserConfig;
use blobman::errors::Result;
use blobman::notify::{ChatterLevel, NotificationBackend};
use blobman::notify::termcolor::TermcolorNotificationBackend;


fn inner(_matches: ArgMatches, _config: UserConfig, nbe: &mut TermcolorNotificationBackend) -> Result<i32> {
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

    // Read in the configuration.

    let config = match UserConfig::open() {
        Ok(c) => c,
        Err(ref e) => {
            // Uh-oh, we couldn't get the configuration. Our main
            // error-printing code requires a 'status' object, which we don't
            // have yet. If we can't even load the config we might really be
            // in trouble, so it seems safest to keep things simple anyway and
            // just use bare stderr without colorization.
            e.dump_uncolorized();
            process::exit(1);
        }
    };

    // Set up colorized output. This comes after the config because you could
    // imagine wanting to be able to configure the colorization (which is
    // something I'd be relatively OK with since it'd only affect the progam
    // UI, not the processing results). Of course, we don’t actually do this
    // just yet.

    let mut nbe = TermcolorNotificationBackend::new(chatter);

    // Now that we've got colorized output, we're to pass off to the inner
    // function ... all so that we can print out the word "error:" in red.
    // This code parallels various bits of the `error_chain` crate.

    process::exit(match inner(matches, config, &mut nbe) {
        Ok(ret) => ret,

        Err(ref e) => {
            nbe.bare_error(e);
            1
        }
    })
}
