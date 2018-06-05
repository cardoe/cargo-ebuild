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
extern crate failure;

use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::Method;
use cargo::core::{Package, PackageSet, Resolve, Workspace};
use cargo::ops;
use cargo::util::{important_paths, CargoResult};
use cargo::Config;
pub use failure::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf; // re-exported to main

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "build")]
    /// Build an ebuild file from a cargo project
    Build {
        #[structopt(long = "ebuild-path", short = "o")]
        ebuild_path: Option<String>,
    },
}

/// Parse cli commands
pub fn run_cargo_ebuild(cmd: Option<Command>) -> Result<(), Error> {
    // If no command is specified run build with default conf
    let cmd = cmd.unwrap_or(Command::Build {
        ebuild_path: None,
    });

    // Here will be the match of the commands, now just example
    match cmd {
        Command::Build { ebuild_path } => build(ebuild_path),
    }
}

/// Finds the root Cargo.toml of the workspace
fn workspace(config: &Config) -> CargoResult<Workspace> {
    let root = important_paths::find_root_manifest_for_wd(&config.cwd())?;
    Workspace::new(&root, config)
}

/// Generates a package registry by using the Cargo.lock or creating one as necessary
fn registry<'a>(config: &'a Config, package: &Package) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources(&[package.package_id().source_id().clone()])?;
    Ok(registry)
}

/// Resolve the packages necessary for the workspace
fn resolve<'a>(
    registry: &mut PackageRegistry<'a>,
    workspace: &Workspace<'a>,
) -> CargoResult<(PackageSet<'a>, Resolve)> {
    // resolve our dependencies
    let (packages, resolve) = ops::resolve_ws(workspace)?;

    // resolve with all features set so we ensure we get all of the depends downloaded
    let resolve = ops::resolve_with_previous(
        registry,
        workspace,
        /* resolve it all */
        Method::Everything,
        /* previous */
        Some(&resolve),
        /* don't avoid any */
        None,
        /* specs */
        &[],
        // register patches
        true,
        // warn
        false,
    )?;

    Ok((packages, resolve))
}

pub fn build(ebuild_path: Option<String>) -> Result<(), Error> {
    // build the crate URIs
    let config = &mut Config::default()?;

    config.configure(
        0,
        Some(false),
        /* color */
        &None,
        /* frozen */
        false,
        /* locked */
        false,
        // unstable flag
        &Vec::new(),
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
    let desc = metadata
        .description
        .as_ref()
        .cloned()
        .unwrap_or_else(|| package.name().to_string());

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
    let path = ebuild_path.map_or_else(
        || PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version())),
        |path| PathBuf::from(path)
        );

    let ebuild_path = if path.is_dir() {
        if !path.exists() {
            return Err(format_err!("No such file or directory"))
        }
        let ebuild_name = PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version()));
        path.join(ebuild_name)
    } else {
        path
    };

    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)?;

    // write the contents out
    write!(
        file,
        include_str!("ebuild.template"),
        description = desc.trim(),
        homepage = homepage.trim(),
        license = license.trim(),
        crates = crates.join(""),
        cargo_ebuild_ver = env!("CARGO_PKG_VERSION"),
        this_year = 1900 + time::now().tm_year
    )?;

    println!("Wrote: {}", ebuild_path.display());

    Ok(())
}
