/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use cargo_metadata::Package;
use itertools::Itertools;
use std::collections::BTreeSet;

pub struct EbuildConfig {
    pub name: String,
    pub version: String,
    pub inherit: Option<String>,
    pub homepage: String,
    pub description: String,
    pub license: String,
    pub restrict: Option<String>,
    pub slot: Option<String>,
    pub keywords: Option<String>,
    pub iuse: Option<String>,
    pub depend: Option<String>,
    pub rdepend: Option<String>,
    pub pdepend: Option<String>,
    pub depend_is_rdepend: bool,
    pub crates: Vec<String>,
}

impl EbuildConfig {
    pub fn from_package(package: Package, crates: Vec<String>, licenses: BTreeSet<String>) -> Self {
        // package description
        let desc = package
            .description
            .as_ref()
            .cloned()
            .unwrap_or_else(|| package.name.clone());

        // package homepage
        let homepage = package.repository.unwrap_or_else(|| {
            String::from("homepage field in Cargo.toml inaccessible to cargo metadata")
        });

        EbuildConfig {
            name: package.name,
            version: package.version.to_string(),
            inherit: None,
            homepage,
            description: desc,
            license: licenses.iter().format(" ").to_string(),
            restrict: None,
            slot: None,
            keywords: None,
            iuse: None,
            depend: None,
            rdepend: None,
            pdepend: None,
            depend_is_rdepend: true,
            crates,
        }
    }
}
