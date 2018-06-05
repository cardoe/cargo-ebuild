/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate structopt;
extern crate env_logger;
extern crate cargo_ebuild;
extern crate log;

use log::Level as LogLevel;
use cargo_ebuild::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cli {
    #[structopt(subcommand)] // the real cargo-ebuild commands
    pub cmd: Option<Command>,

    /// Prevent cargo-ebuild and cargo to use stdout
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, -vvvv)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,
}

fn main() -> Result<(), Error> {
    setup_panic!();

    let args = Cli::from_args();

    env_logger::Builder::new()
        .filter(
            None,
            match args.verbosity {
                0 => LogLevel::Error,
                1 => LogLevel::Warn,
                2 => LogLevel::Info,
                3 => LogLevel::Debug,
                _ => LogLevel::Trace,
            }.to_level_filter(),
        )
        .try_init()?;


    run_cargo_ebuild(args.cmd)
}
