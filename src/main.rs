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
use cargo_ebuild::run;
use std::process;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Args {
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}

#[derive(StructOpt, Debug)]
#[structopt(bin_name = "cargo")]
enum Opt {
    #[structopt(
        name = "ebuild",
        raw(
            setting = "AppSettings::UnifiedHelpMessage",
            setting = "AppSettings::DeriveDisplayOrder",
            setting = "AppSettings::DontCollapseArgsInUsage"
        )
    )]
    /// Generates an ebuild for a given Cargo project
    Ebuild(Args),
}

fn main() {
    let Opt::Ebuild(opt) = Opt::from_args();

    // run the actual code
    if let Err(e) = run(opt.verbose as u32, opt.quiet) {
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
