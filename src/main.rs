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
extern crate quicli;
#[macro_use]
extern crate log;
#[macro_use]
extern crate human_panic;
extern crate cargo_ebuild;

use cargo_ebuild::*;
use quicli::prelude::StructOpt;
use std::fs::OpenOptions;
use std::path::PathBuf;

main!(|opt: Opt, log_level: verbose| {
    setup_panic!();

    let ebuild = match ebuild_from_cargo(opt) {
        Ok(ebuild) => ebuild,
        Err(err) => {
            error!("{}", err);
            panic!("{}", err);
        }
    };

    debug!("Generated {:#?}", ebuild);

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild",
                                            ebuild.name(), ebuild.version()));

    debug!("Ebuild path: {:?}", ebuild_path);

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)
        .expect("Failed to open ebuild file");

    // Write the ebuild
    match ebuild.write(&mut file) {
        Ok(_) => println!("Wrote: {}", ebuild_path.display()),
        Err(err) => error!("{:?}", err),
    };
});
