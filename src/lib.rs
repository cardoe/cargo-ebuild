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
#[macro_use]
extern crate serde_derive;
extern crate time;

use cargo::{Config, CliResult};
use cargo::core::{Package, PackageSet, Resolve, Workspace};
use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::Method;
use cargo::ops;
use cargo::util::{important_paths, CargoResult, CargoResultExt};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Options {
    flag_verbose: u32,
    flag_quiet: Option<bool>,
}

/// Finds the root Cargo.toml of the workspace
fn workspace(config: &Config, manifest_path: Option<String>) -> CargoResult<Workspace> {
    let root = important_paths::find_root_manifest_for_wd(manifest_path, config.cwd())?;
    Workspace::new(&root, config)
}

/// Generates a package registry by using the Cargo.lock or creating one as necessary
fn registry<'a>(config: &'a Config, package: &Package) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config)?;
    registry
        .add_sources(&[package.package_id().source_id().clone()])?;
    Ok(registry)
}

/// Resolve the packages necessary for the workspace
fn resolve<'a>(registry: &mut PackageRegistry,
               workspace: &'a Workspace)
               -> CargoResult<(PackageSet<'a>, Resolve)> {
    // resolve our dependencies
    let (packages, resolve) = ops::resolve_ws(workspace)?;

    // resolve with all features set so we ensure we get all of the depends downloaded
    let resolve = ops::resolve_with_previous(registry,
                                             workspace,
                                             /* resolve it all */
                                             Method::Everything,
                                             /* previous */
                                             Some(&resolve),
                                             /* don't avoid any */
                                             None,
                                             /* specs */
                                             &[])?;

    Ok((packages, resolve))
}

pub fn real_main(options: Options, config: &Config) -> CliResult {
    config
        .configure(options.flag_verbose,
                   options.flag_quiet,
                   /* color */
                   &None,
                   /* frozen */
                   false,
                   /* locked */
                   false)?;

    // Load the workspace and current package
    let workspace = workspace(config, None)?;
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
    let desc = metadata
        .description
        .as_ref()
        .cloned()
        .unwrap_or_else(|| String::from(package.name()));

    // package homepage
    let homepage =
        metadata.homepage.as_ref().cloned().unwrap_or(metadata
                                                          .repository
                                                          .as_ref()
                                                          .cloned()
                                                          .unwrap_or_else(|| String::from("")));

    let license = metadata
        .license
        .as_ref()
        .cloned()
        .unwrap_or_else(|| String::from("unknown license"));

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version()));

    // Open the file where we'll write the ebuild
    let mut file = try!(OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&ebuild_path)
                            .chain_err(|| "failed to create ebuild"));

    // write the contents out
    try!(write!(file,
                include_str!("ebuild.template"),
                description = desc.trim(),
                homepage = homepage.trim(),
                license = license.trim(),
                crates = crates.join(""),
                cargo_ebuild_ver = env!("CARGO_PKG_VERSION"),
                this_year = 1900 + time::now().tm_year,
                ).chain_err(|| "unable to write ebuild to disk"));

    println!("Wrote: {}", ebuild_path.display());


    Ok(())
}
