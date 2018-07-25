/*
 * Copyright 2016-2018 Doug Goldstein <cardoe@cardoe.com>
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
*/

extern crate cargo_metadata;
extern crate time;
extern crate toml;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub use failure::Error; // re-exported to main
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;

pub fn ebuild(ebuild_path: Option<String>, manifest_path: Option<String>) -> Result<(), Error> {
    let manifest = manifest_path.map_or_else(
        || env::current_dir().unwrap().join("Cargo.toml"),
        |path| Path::new(&path).to_path_buf(),
    );

    if !manifest.is_file() {
        return Err(format_err!(
            "Cargo manifest not found at: {}",
            manifest.display()
        ));
    }

    let metadata = cargo_metadata::metadata_deps(Some(&manifest), true)
        .map_err(|err| format_err!("Error while access cargo metadata: {}", err))?;

    let resolve = metadata
        .resolve
        .ok_or_else(|| format_err!("No project dependences"))?
        .nodes;

    let mut git_crates = Vec::new();

    // build the crates the package needs
    let mut crates = resolve
        .iter()
        .cloned()
        .filter_map(|pkg| {
            let infopkg: Vec<&str> = pkg.id.split(' ').collect();
            if infopkg.len() != 3 {
                None
            } else if infopkg[2].starts_with("(git") {
                git_crates.push(infopkg[2][1..infopkg[2].len()].to_string());
                None
            } else {
                Some(format!("{}-{}", infopkg[0], infopkg[1]))
            }
        })
        .collect::<Vec<String>>();

    // sort the crates
    crates.sort();

    let mut string = String::new();
    File::open(&manifest)?.read_to_string(&mut string)?;

    let parsed_manifest = string.parse::<Value>()?;
    let table = &parsed_manifest
        .as_table()
        .ok_or_else(|| format_err!("Cargo manifest does not contain a toml table"))?;
    let package = table
        .get("package")
        .ok_or_else(|| format_err!("Field \"package\" is missing in Cargo manifest"))?;
    let name = read_from_manifest(package, &"name").unwrap_or_else(|| {
        warn!("Not found package's name field in Cargo.toml. Manual setup needed");
        String::from("")
    });

    let mut license = read_from_manifest(package, &"license").unwrap_or_else(|| {
        warn!("Not found package's license field in Cargo.toml. Look for license-file.");

        read_from_manifest(package, &"license-file").unwrap_or_else(|| {
            warn!("Not found package's license-file field in Cargo.toml. Manual setup needed");
            String::from("")
        })
    });

    // convert cargo format to portage
    if license.contains('/') {
        license = license.as_str().split('/').map(|f| f.trim()).collect::<Vec<&str>>().join(" | ");
    } else if license.contains("AND") || license.contains("OR") {
        license = license.replace("AND", "&&");
        license = license.replace("OR", "||");
    }

    let description = read_from_manifest(package, &"description").unwrap_or_else(|| {
        warn!("Not found package's description field in Cargo.toml. Used package's name");
        name.clone()
    });
    let repository = read_from_manifest(package, &"repository").unwrap_or_else(|| {
        warn!("Not found package's repository field in Cargo.toml. Manual setup needed");
        String::from("")
    });
    let homepage = read_from_manifest(package, &"homepage").unwrap_or_else(|| {
        warn!("Not found package's name field in Cargo.toml. Manual setup needed");
        repository
    });
    let version = read_from_manifest(package, &"version").unwrap_or_else(|| {
        warn!("Not found package's version field in Cargo.toml. Manual setup needed");
        String::from("")
    });

    // look for features
    let features = match metadata.packages.iter().filter(|pkg| {
        name == pkg.name
    }).collect::<Vec<&cargo_metadata::Package>>().pop() {
        Some(target) => target.features.keys().map(|key| key.to_string()).collect(),
        None => Vec::new(),
    };

    debug!("For {} found features {:?}", name, features);

    // build up the ebuild path
    let path = match ebuild_path {
        Some(path_arg) => PathBuf::from(path_arg),
        None => std::env::current_dir().unwrap(),
    };

    let ebuild_path = if path.is_dir() {
        if !path.exists() {
            return Err(format_err!("No such file or directory"));
        }
        let ebuild_name = PathBuf::from(format!("{}-{}.ebuild", name, version));
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
    writeln!(
        file,
        include_str!("ebuild.template"),
        description = description.trim(),
        homepage = homepage.trim(),
        license = license.trim(),
        crates = crates.join("\n"),
        features = features.join(" "),
        cargo_ebuild_ver = &env!("CARGO_PKG_VERSION"),
        this_year = 1900 + time::now().tm_year
    )?;

    println!("Wrote: {}", ebuild_path.display());

    Ok(())
}

fn read_from_manifest(package: &Value, query: &str) -> Option<String> {
    package.get(query)?.clone().try_into::<String>().ok()
}
