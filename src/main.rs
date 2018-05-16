/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

extern crate cargo;
extern crate time;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate quicli;
#[macro_use]
extern crate log;
#[macro_use]
extern crate human_panic;

pub mod cargo_utils;
pub mod core;

use cargo::Config;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use core::*;


#[derive(StructOpt, Debug)]
#[structopt(name = "cargo ebuild")]
// tmp public
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u32,

    /// No output printed to stdout
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Arguments passed to cargo
    #[structopt(short = "f", long = "unstable-flags", parse(from_str))]
    unstable_flags: Vec<String>,
}

main!(|opt: Opt, log_level: verbose| {
    setup_panic!();

    let mut config = Config::default().unwrap();

    let ebuild = ebuild_from_cargo(opt, &mut config).unwrap();

    info!("{:?}", ebuild);

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", ebuild.name, ebuild.version));

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)
        .unwrap();

    write!(file,
        include_str!("ebuild.template"),
        description = ebuild.description,
        homepage = ebuild.homepage,
        license = ebuild.license,
        crates = ebuild.crates,
        cargo_ebuild_ver = ebuild.version,
        this_year = ebuild.year
        ).expect("Error during ebuild file writing");

    println!("Wrote: {}", ebuild_path.display());
});

