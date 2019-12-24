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
extern crate structopt;

use cargo::util::CliError;
use cargo_ebuild::run;
use std::path::PathBuf;
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

    #[structopt(name = "PATH", long = "manifest-path", parse(from_os_str))]
    /// Path to Cargo.toml.
    manifest_path: Option<PathBuf>,
}

#[structopt(
    name = "cargo-ebuild",
    bin_name = "cargo",
    author,
    about = "Generates an ebuild for a given Cargo project",
    global_settings(&[AppSettings::ColoredHelp])
)]
#[derive(StructOpt, Debug)]
enum Opt {
    #[structopt(name = "ebuild")]
    /// Generates an ebuild for a given Cargo project
    Ebuild(Args),
}

fn main() {
    let Opt::Ebuild(opt) = Opt::from_args();

    // run the actual code
    if let Err(e) = run(opt.verbose as u32, opt.quiet, opt.manifest_path) {
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
