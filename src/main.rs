/*
 * Copyright 2016-2017 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

extern crate cargo;
extern crate cargo_ebuild;
#[macro_use]
extern crate structopt;

use cargo::util::CliError;
use cargo::CliResult;
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
    Ebuild,
}

/// Parse cli commands
pub fn run(cmd: Option<Command>, verbose: u32, quiet: bool) -> CliResult {
    // If no command is specified run build with default conf
    let cmd = cmd.unwrap_or(Command::Ebuild);

    // Here will be the match of the commands, now just example
    match cmd {
        Command::Ebuild => cargo_ebuild::ebuild(verbose, quiet),
    }
}

fn main() {
    let opt = Opt::from_args();

    // run the actual code
    if let Err(e) = run(opt.cmd, opt.verbose as u32, opt.quiet) {
        // break apart the error
        let CliError {
            error,
            exit_code,
            unknown: _unknown,
        } = e;
        // display a msg if we got one
        if let Some(msg) = error {
            eprintln!("{}", msg);
        }
        // exit appropriately
        process::exit(exit_code);
    }
}
