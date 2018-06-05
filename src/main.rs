/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

extern crate cargo_ebuild;
extern crate cargo;
#[macro_use]
extern crate structopt;

use cargo::Config;
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

fn main() {
    let args = Cli::from_args();
    let mut config = Config::default().unwrap();
    let result = run_cargo_ebuild(&mut config, args.cmd);
    if let Err(e) = result {
        cargo::exit_with_error(e, &mut *config.shell());
    }
}
