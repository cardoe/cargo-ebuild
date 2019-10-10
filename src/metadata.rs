/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use cargo::core::Package;

pub struct EbuildConfig {
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
    pub fn from_package(package: &Package, crates: Vec<String>) -> Self {
        // root package metadata
        let metadata = package.manifest().metadata();

        // package description
        let desc = metadata
            .description
            .as_ref()
            .cloned()
            .unwrap_or_else(|| package.name().to_string());

        // package homepage
        let homepage = metadata.homepage.as_ref().cloned().unwrap_or_else(|| {
            metadata
                .repository
                .as_ref()
                .cloned()
                .unwrap_or_else(|| String::from(""))
        });

        let license = metadata
            .license
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from("unknown license"));

        EbuildConfig {
            inherit: None,
            homepage,
            description: desc,
            license,
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
