// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

use app_dirs;
use std::{convert, io};
use toml;


error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        AppDirs(app_dirs::AppDirsError);
        Io(io::Error);
        TomlDe(toml::de::Error);
    }
}


#[macro_export]
macro_rules! ctry {
    ($op:expr ; $( $chain_fmt_args:expr ),*) => {
        $op.chain_err(|| format!($( $chain_fmt_args ),*))?
    }
}

impl convert::From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, format!("{}", err))
    }
}


impl Error {
    /// Write the information contained in this object to standard error in a
    /// somewhat user-friendly form.
    ///
    /// The `error_chain` crate provides a Display impl for its Error objects
    /// that ought to provide this functionality, but I have had enormous
    /// trouble being able to use it. So instead we emulate their code. The
    /// CLI program provides very similar code that produces similar output
    /// but with fancy colorization.
    pub fn dump_uncolorized(&self) {
        let mut prefix = "error:";

        for item in self.iter() {
            eprintln!("{} {}", prefix, item);
            prefix = "caused by:";
        }

        if let Some(backtrace) = self.backtrace() {
            eprintln!("debugging: backtrace follows:");
            eprintln!("{:?}", backtrace);
        }
    }
}
