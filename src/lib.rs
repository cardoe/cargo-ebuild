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

use cargo::core::Workspace;
use cargo::util::{important_paths, CargoResult};
use cargo::{CliResult, Config};
use failure::format_err;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use metadata::EbuildConfig;

/// Finds the root Cargo.toml of the workspace
fn workspace(config: &Config, manifest: impl AsRef<Path>) -> CargoResult<Workspace> {
    let root = important_paths::find_root_manifest_for_wd(manifest.as_ref())?;
    Workspace::new(&root, config)
}

pub fn run(verbose: u32, quiet: bool, manifest_path: Option<PathBuf>) -> CliResult {
    let mut cmd = cargo_metadata::MetadataCommand::new();

    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd
        .exec()
        .map_err(|e| format_err!("cargo metadata failed: {}", e))?;

    let mut crates = Vec::with_capacity(metadata.packages.len());
    for pkg in metadata.packages {
        crates.push(format!("{}-{}\n", pkg.name, pkg.version));
    }

    // sort the crates
    crates.sort();

    // create a default Cargo config
    let mut config = Config::default()?;

    config.configure(
        verbose,
        Some(quiet),
        /* color */
        &None,
        /* frozen */
        false,
        /* locked */
        false,
        /* offline */
        false,
        /* target dir */
        &None,
        /* unstable flags */
        &[],
    )?;

    // Load the workspace and current package
    let workspace = workspace(&config, &metadata.workspace_root)?;
    let package = workspace.current()?;

    let ebuild_data = EbuildConfig::from_package(package, crates);

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version()));

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
