// Copyright 2017-2019 Peter Williams and collaborators
// Licensed under the MIT License.

//! The entrypoint of the main "blobman" CLI command.

use blobman::{
    collection::Collection,
    config::UserConfig,
    errors::Result,
    notify::termcolor::TermcolorNotificationBackend,
    notify::{BufferingNotificationBackend, ChatterLevel, NotificationBackend},
    retrieval::url::UrlRetrieval,
};
use std::process;
use structopt::StructOpt;
use tokio::runtime::Runtime;
use toml::{self, value::Table};

/// StructOpt parameters for the "cat" subcommand.
#[derive(Debug, StructOpt)]
pub struct BlobmanCatOptions {
    #[structopt(help = "The name of the blob to stream")]
    name: String,
}

impl BlobmanCatOptions {
    fn cli(self, _config: &UserConfig, _nbe: &mut (dyn NotificationBackend + Send)) -> Result<i32> {
        // let mut sess = blobman::Session::new(&config, nbe)?;
        // let mut bstream = sess.open_blob(&self.name)?;
        // let mut stdout = io::stdout();
        // io::copy(&mut bstream, &mut stdout)?;
        // stdout.flush()?; // note: empirically, this is necessary
        Ok(0)
    }
}

/// StructOpt parameters for the "fetch" subcommand.
#[derive(Debug, StructOpt)]
pub struct BlobmanFetchOptions {
    #[structopt(
        short = "n",
        help = "The name to use for the fetched blob [default: derived from URL]"
    )]
    name: Option<String>,

    #[structopt(
        short = "m",
        help = "How to act if the blob is already registered",
        possible_values = blobman::IngestMode::stringifications(),
        default_value = "update",
    )]
    mode: String,

    #[structopt(help = "The URL to download")]
    url: String,
}

impl BlobmanFetchOptions {
    fn cli(self, config: &UserConfig, nbe: &mut (dyn NotificationBackend + Send)) -> Result<i32> {
        // let mode = self.mode.parse()?;
        let mut sess = blobman::Session::new(&config, nbe)?;

        if let None = sess.get_collection("www") {
            let retr = UrlRetrieval::new();
            let mut c = Collection::new("www", Box::new(retr));
            c.set_keys(&["url"]);
            sess.insert_collection(c);
        }

        let mut item_spec = Table::new();
        item_spec.insert("url".to_owned(), toml::Value::String(self.url.clone()));

        let rt = Runtime::new()?;
        rt.block_on(sess.insert_item("www", item_spec))?;

        sess.rewrite_manifest()?;
        Ok(0)
    }
}

/// StructOpt parameters for the "provide" subcommand.
#[derive(Debug, StructOpt)]
pub struct BlobmanProvideOptions {
    #[structopt(help = "The name of the blob to provide")]
    name: String,
}

impl BlobmanProvideOptions {
    fn cli(self, _config: &UserConfig, _nbe: &mut (dyn NotificationBackend + Send)) -> Result<i32> {
        // let mut sess = blobman::Session::new(&config, nbe)?;
        // sess.provide_blob(&self.name)?;
        Ok(0)
    }
}

/// The main StructOpt type for dispatching subcommands.
#[derive(Debug, StructOpt)]
pub enum BlobmanSubcommand {
    #[structopt(name = "cat")]
    /// Stream blob data to standard output
    Cat(BlobmanCatOptions),

    #[structopt(name = "fetch")]
    /// Download and ingest a file
    Fetch(BlobmanFetchOptions),

    #[structopt(name = "provide")]
    /// Make a file corresponding to the named blob
    Provide(BlobmanProvideOptions),
}

/// The main StructOpt argument dispatcher.
#[derive(Debug, StructOpt)]
#[structopt(name = "blobman", about = "Manage data files.")]
pub struct BlobmanCli {
    #[structopt(subcommand)]
    command: BlobmanSubcommand,

    #[structopt(
        short = "c",
        help = "How much chatter to print when running",
        default_value = "default",
        possible_values = &["default", "minimal"],
    )]
    chatter: String,
}

impl BlobmanCli {
    fn cli(self, config: UserConfig, nbe: &mut (dyn NotificationBackend + Send)) -> Result<i32> {
        match self.command {
            BlobmanSubcommand::Cat(opts) => opts.cli(&config, nbe),
            BlobmanSubcommand::Fetch(opts) => opts.cli(&config, nbe),
            BlobmanSubcommand::Provide(opts) => opts.cli(&config, nbe),
        }
    }
}

fn main() {
    let invocation = BlobmanCli::from_args();

    let chatter = match invocation.chatter.as_ref() {
        "default" => ChatterLevel::Normal,
        "minimal" => ChatterLevel::Minimal,
        _ => unreachable!(),
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

    process::exit(match invocation.cli(config, &mut nbe) {
        Ok(ret) => ret,

        Err(ref e) => {
            nbe.bare_error(e);
            1
        }
    })
}
