/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

mod metadata;

use anyhow::{format_err, Context, Result};
use cargo_lock::Lockfile;
use cargo_metadata::MetadataCommand;
use cargo_metadata::CargoOpt;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use metadata::EbuildConfig;

fn parse_license<'a>(lic_str: &'a str) -> Vec<&'a str> {
    lic_str
        .split('/')
        .flat_map(|l| l.split(" OR "))
        .flat_map(|l| l.split(" AND "))
        .map(str::trim)
        .collect()
}

fn generate_lockfile(manifest_path: Option<PathBuf>) -> Result<()> {
    let cargo = std::env::var("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("cargo"));

    let mut lock_cmd = Command::new(cargo);
    lock_cmd.arg("generate-lockfile");

    if let Some(path) = manifest_path.as_ref() {
        lock_cmd.arg("--manifest-path");
        lock_cmd.arg(path.as_os_str());
    }

    let lock_output = lock_cmd.output()?;

    if !lock_output.status.success() {
        let stderr = String::from_utf8_lossy(&lock_output.stderr);
        return Err(format_err!("unable to generate lockfile:\n{}", stderr));
    }

    Ok(())
}

pub fn gen_ebuild_data(manifest_path: Option<PathBuf>) -> Result<EbuildConfig> {
    let mut cmd = MetadataCommand::new();

    cmd.features(CargoOpt::AllFeatures);

    if let Some(path) = manifest_path.as_ref() {
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

    let mut licenses = BTreeSet::new();
    let mut crates = Vec::new();
    let mut root_pkg = None;

    for pkg in metadata.packages {
        if &pkg.id == root {
            root_pkg = Some(pkg.clone());
        }

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

    let lockfile_path = metadata.workspace_root.join("Cargo.lock");

    // Generate lockfile if it doesn't exists
    if std::fs::metadata(&lockfile_path).is_err() {
        generate_lockfile(manifest_path)?;
    }

    // Check for packages that must be fetched from default registry
    let lockfile = Lockfile::load(lockfile_path)?;

    for pkg in lockfile.packages {
        if let Some(src) = pkg.source {
            if src.is_default_registry() {
                crates.push(format!("{}-{}\n", pkg.name, pkg.version));
            }
        }
    }

    Ok(EbuildConfig::from_package(root_pkg, crates, licenses))
}

pub fn write_ebuild(ebuild_data: EbuildConfig, ebuild_path: impl AsRef<Path>) -> Result<()> {
    // Open the file where we'll write the ebuild
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ebuild_path)
        .context(format!(
            "Unable to create {}",
            ebuild_path.as_ref().display()
        ))?;

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
    .context(format!(
        "Failed to write to {}",
        ebuild_path.as_ref().display()
    ))
}
