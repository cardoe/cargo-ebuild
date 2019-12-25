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

mod metadata;

use failure::format_err;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use metadata::EbuildConfig;

fn parse_license<'a>(lic_str: &'a str) -> Vec<&'a str> {
    lic_str
        .split('/')
        .flat_map(|l| l.split(" OR "))
        .flat_map(|l| l.split(" AND "))
        .map(str::trim)
        .collect()
}

pub fn run(verbose: u32, quiet: bool, manifest_path: Option<PathBuf>) -> CliResult {
    let mut cmd = cargo_metadata::MetadataCommand::new();

    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd
        .exec()
        .map_err(|e| format_err!("cargo metadata failed: {}", e))?;

    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| format_err!("cargo metadata did not resolve the depend graph"))?;
    let root = resolve
        .root
        .as_ref()
        .ok_or_else(|| format_err!("cargo metadata failed to resolve the root package"))?;

    let mut crates = Vec::with_capacity(metadata.packages.len());
    let mut licenses = BTreeSet::new();
    let mut root_pkg = None;
    for pkg in metadata.packages {
        if &pkg.id == root {
            root_pkg = Some(pkg.clone());
        }

        crates.push(format!("{}-{}\n", pkg.name, pkg.version));

        if let Some(lic_list) = pkg.license.as_ref().map(|l| parse_license(&l)) {
            for lic in lic_list.iter() {
                licenses.insert(lic.to_string());
            }
        }
        if pkg.license_file.is_some() {
            println!("WARNING: {} uses a license-file, not handled", pkg.name);
        }
    }

    let root_pkg = root_pkg
        .ok_or_else(|| format_err!("unable to determine package to generate ebuild for"))?;

    // sort the crates
    crates.sort();

    let root_pkg_name_ver = format!("{}-{}", root_pkg.name, root_pkg.version);

    let ebuild_data = EbuildConfig::from_package(root_pkg, crates, licenses);

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}.ebuild", root_pkg_name_ver));

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)
        .expect("failed to create ebuild");

    // write the contents out
    write!(
        file,
        include_str!("ebuild.template"),
        description = ebuild_data.description.trim(),
        homepage = ebuild_data.homepage.trim(),
        license = ebuild_data.license.trim(),
        crates = ebuild_data.crates.join(""),
        cargo_ebuild_ver = env!("CARGO_PKG_VERSION"),
        this_year = 1900 + time::now().tm_year,
    )
    .expect("unable to write ebuild to disk");

    println!("Wrote: {}", ebuild_path.display());

    Ok(())
}
