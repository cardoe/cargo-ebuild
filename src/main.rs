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

use cargo::{CliResult, Config};
use cargo_utils::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo ebuild")]
struct Opt {
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

    if let Err(e) = real_main(opt, &mut config) {
        error!("{:?}", e);
    };
});

fn real_main(options: Opt, config: &mut Config) -> CliResult {
    config.configure(
        options.verbose,
        Some(options.quiet),
        // color
        &None,
        // frozen
        false,
        // locked
        false,
        // unstable flag
        &options.unstable_flags,
    )?;

    // Load the workspace and current package
    let workspace = workspace(config)?;
    let package = workspace.current()?;

    // Resolve all dependencies (generate or use Cargo.lock as necessary)
    let mut registry = registry(config, &package)?;
    let resolve = resolve(&mut registry, &workspace)?;

    // build the crates the package needs
    let mut crates = resolve
        .1
        .iter()
        .map(|pkg| format!("{}-{}\n", pkg.name(), pkg.version()))
        .collect::<Vec<String>>();

    // sort the crates
    crates.sort();

    // root package metadata
    let metadata = package.manifest().metadata();

    // package description
    let desc = metadata.description.as_ref().cloned().unwrap();
    // TODO: now package.name is InternedString
    // .unwrap_or_else(|| String::from(package.name()));

    // package homepage
    let homepage = metadata.homepage.as_ref().cloned().unwrap_or(
        metadata
            .repository
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from("")),
    );

    let license = metadata
        .license
        .as_ref()
        .cloned()
        .unwrap_or_else(|| String::from("unknown license"));

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version()));

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)
        .unwrap();

    // write the contents out
    write!(
        file,
        include_str!("ebuild.template"),
        description = desc.trim(),
        homepage = homepage.trim(),
        license = license.trim(),
        crates = crates.join(""),
        cargo_ebuild_ver = env!("CARGO_PKG_VERSION"),
        this_year = 1900 + time::now().tm_year,
    ).expect("Error during ebuild file writing");

    println!("Wrote: {}", ebuild_path.display());

    Ok(())
}
