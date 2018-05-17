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
extern crate failure;
extern crate quicli;
#[macro_use]
extern crate human_panic;

use cargo_ebuild::run_cargo_ebuild;
use cargo_ebuild::Cli;
use failure::Error;
use quicli::prelude::*;
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() -> std::result::Result<(), Error> {
    let args = Cli::from_args();
    let is_quiet = args.quiet;

    setup_panic!();

    LoggerBuiler::new()
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

    // call the real cargo_ebuild
    let ebuild = run_cargo_ebuild(args)?;
    debug!("Generated ebuild {:#?}", ebuild);

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", ebuild.name(), ebuild.version()));
    debug!("Ebuild file: {:?}", ebuild_path);

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)?;

    // Write the ebuild
    ebuild.write(&mut file)?;
    if is_quiet {
        info!("Wrote: {}", ebuild_path.display());
    } else {
        println!("Wrote: {}", ebuild_path.display());
    }

    Ok(())
}
