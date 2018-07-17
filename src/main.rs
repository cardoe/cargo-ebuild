/*
 * Copyright 2016-2017 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

extern crate cargo_ebuild;
#[macro_use]
extern crate structopt;

use cargo_ebuild::*;
use std::process;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo", raw(setting = "AppSettings::ColoredHelp"))]
struct Opt {
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
    /// Generates an ebuild for a given Cargo project
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "ebuild")]
    /// Build an ebuild file from a cargo project
    Ebuild {
        #[structopt(long = "ebuild-path", short = "o")]
        ebuild_path: Option<String>,
        #[structopt(long = "manifest-path", short = "m")]
        manifest_path: Option<String>,
    },
}

/// Parse cli commands
pub fn run(cmd: Option<Command>) -> Result<(), Error> {
    // If no command is specified run build with default conf
    let cmd = cmd.unwrap_or(Command::Ebuild {
        ebuild_path: None,
        manifest_path: None,
    });

    // Here will be the match of the commands, now just example
    match cmd {
        Command::Ebuild {
            ebuild_path,
            manifest_path,
        } => ebuild(ebuild_path, manifest_path),
    }
}

fn main() {
    let opt = Opt::from_args();

    // run the actual code
    if let Err(error) = run(opt.cmd) {
        // display a msg if we got one
        println!("{:?}", error);
        // exit appropriately
        process::exit(1);
    }
}
