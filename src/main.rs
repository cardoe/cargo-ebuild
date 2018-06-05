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

use cargo::Config;
use std::env;

const USAGE: &'static str = r#"
Create an ebuild for a project

Usage:
    cargo ebuild [options]

Options:
    -h, --help          Print this message
    -v, --verbose       Use verbose output
    -q, --quiet         No output printed to stdout
"#;

fn main() {
    let mut config = Config::default().unwrap();
    let args = env::args().collect::<Vec<_>>();
    let result = cargo_ebuild::real_main(&mut config);
    if let Err(e) = result {
        cargo::exit_with_error(e, &mut *config.shell());
    }
}

