// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

#[macro_use] extern crate blobman;
extern crate clap;

use blobman::config::UserConfig;
use blobman::errors::Result;
use blobman::notify::{BufferingNotificationBackend, ChatterLevel};
use blobman::notify::termcolor::TermcolorNotificationBackend;
use clap::{Arg, ArgMatches, App, SubCommand};
use std::process;


fn inner(matches: ArgMatches, config: UserConfig, nbe: &mut TermcolorNotificationBackend) -> Result<i32> {
    if let Some(fetch_m) = matches.subcommand_matches("fetch") {
        let mut sess = blobman::Session::new(&config, nbe)?;
        sess.ingest_from_url(fetch_m.value_of("URL").unwrap())?;
        sess.rewrite_manifest()?;
    } else {
        return err_msg!("you must specify a subcommand; try \"blobman help\"");
    }

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
        .subcommand(SubCommand::with_name("fetch")
                    .about("download and ingest a file")
                    .arg(Arg::with_name("URL")
                         .help("The URL to download.")
                         .required(true)
                         .index(1)))
        .get_matches ();

    let chatter = match matches.value_of("chatter_level").unwrap() {
        "default" => ChatterLevel::Normal,
        "minimal" => ChatterLevel::Minimal,
        _ => unreachable!()
    };

    // Read in the configuration. We want to make it possible to decide
    // whether to emit colorized output based on a configuration setting,
    // which means we can't emit notifications just yet; therefore we buffer
    // them.

    let mut temp_nbe = BufferingNotificationBackend::new();

    let config = match UserConfig::open(&mut temp_nbe) {
        Ok(c) => c,
        Err(ref e) => {
            // Uh-oh, we couldn't get the configuration. Our main
            // error-printing code requires a 'status' object, which we don't
            // have yet. If we can't even load the config we might really be
            // in trouble, so it seems safest to keep things simple anyway and
            // just use bare stderr without colorization.
            //
            // NOTE: if anything gets logged to temp_nbe, it gets swallowed.
            eprintln!("fatal: error while reading user configuration file");
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
    temp_nbe.drain(&mut nbe);

    // Now that we've got colorized output, we're ready to pass off to the
    // inner function ... all so that we can print out the word "error:" in
    // red. This code parallels various bits of the `error_chain` crate.

    process::exit(match inner(matches, config, &mut nbe) {
        Ok(ret) => ret,

        Err(ref e) => {
            nbe.bare_error(e);
            1
        }
    })
}
