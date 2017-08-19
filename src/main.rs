// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

#[macro_use] extern crate blobman;
extern crate clap;

use blobman::config::UserConfig;
use blobman::errors::Result;
use blobman::notify::{BufferingNotificationBackend, ChatterLevel};
use blobman::notify::termcolor::TermcolorNotificationBackend;
use clap::{Arg, ArgMatches, App, SubCommand};
use std::io::{self, Write};
use std::process;


fn inner(matches: ArgMatches, config: UserConfig, nbe: &mut TermcolorNotificationBackend) -> Result<i32> {
    if let Some(cat_m) = matches.subcommand_matches("cat") {
        let mut sess = blobman::Session::new(&config, nbe)?;
        let mut bstream = sess.open_blob(cat_m.value_of("NAME").unwrap())?;
        let mut stdout = io::stdout();
        io::copy(&mut bstream, &mut stdout)?;
        stdout.flush()?; // note: empirically, this is necessary
    } else if let Some(fetch_m) = matches.subcommand_matches("fetch") {
        let mode = fetch_m.value_of("MODE").unwrap().parse()?;
        let mut sess = blobman::Session::new(&config, nbe)?;
        sess.ingest_from_url(mode, fetch_m.value_of("URL").unwrap(), fetch_m.value_of("name"))?;
        sess.rewrite_manifest()?;
    } else if let Some(provide_m) = matches.subcommand_matches("provide") {
        let mut sess = blobman::Session::new(&config, nbe)?;
        sess.provide_blob(provide_m.value_of("NAME").unwrap())?;
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
             .help("How much chatter to print when running")
             .possible_values(&["default", "minimal"])
             .default_value("default"))
        .subcommand(SubCommand::with_name("cat")
                    .about("Stream blob data to standard output")
                    .arg(Arg::with_name("NAME")
                         .help("The name of the blob to stream.")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("fetch")
                    .about("Download and ingest a file")
                    .arg(Arg::with_name("name")
                         .long("name")
                         .short("n")
                         .value_name("NAME")
                         .help("The name to use for the fetched blob (default: derived from URL)."))
                    .arg(Arg::with_name("MODE")
                         .long("mode")
                         .short("m")
                         .value_name("MODE")
                         .help("How to act if the blob is already registered")
                         .possible_values(blobman::IngestMode::stringifications())
                         .default_value("update"))
                    .arg(Arg::with_name("URL")
                         .help("The URL to download.")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("provide")
                    .about("Make a file corresponding to the named blob")
                    .arg(Arg::with_name("NAME")
                         .help("The name of the blob to provide.")
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
