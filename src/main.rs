extern crate cargo;
extern crate rustc_serialize;

use cargo::{Config, CliError, CliResult};
use cargo::core::Package;
use cargo::core::registry::PackageRegistry;
use cargo::ops;
use cargo::util::important_paths;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(RustcDecodable)]
struct Options {
    flag_verbose: bool,
    flag_quiet: bool,
}

fn main() {
    cargo::execute_main_without_stdin(real_main,
                                      false,
                                      r#"
Create an ebuild for a project

Usage:
    cargo ebuild [options]

Options:
    -h, --help          Print this message
    -v, --verbose       Use verbose output
    -q, --quiet         No output printed to stdout
"#)
}

fn real_main(options: Options, config: &Config) -> CliResult<Option<()>> {
    try!(config.shell().set_verbosity(options.flag_verbose, options.flag_quiet));

    // Load the root package
    let root = try!(important_paths::find_root_manifest_for_wd(None, config.cwd()));
    let package = try!(Package::for_path(&root, config));

    // Resolve all dependencies (generate or use Cargo.lock as necessary)
    let mut registry = PackageRegistry::new(config);
    try!(registry.add_sources(&[package.package_id().source_id().clone()]));
    let resolve = try!(ops::resolve_pkg(&mut registry, &package, config));

    // build the crates the package needs
    let mut crates = resolve.iter()
        .map(|pkg| format!("{}-{}\n", pkg.name(), pkg.version()))
        .collect::<Vec<String>>();

    // sort the crates
    crates.sort();

    // root package metadata
    let metadata = package.manifest().metadata();

    // package description
    let desc = metadata.description
        .as_ref()
        .cloned()
        .unwrap_or_else(|| String::from(package.name()));

    // package homepage
    let homepage = metadata.homepage
        .as_ref()
        .cloned()
        .unwrap_or(metadata.repository
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from("")));

    let license = metadata.license
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
        .map_err(|err| {
            CliError::new(&format!("failed to create ebuild: {}", err.description()),
                          1)
        }));

    // write the contents out
    try!(write!(file,
                include_str!("ebuild.template"),
                description = desc.trim(),
                homepage = homepage.trim(),
                license = license.trim(),
                crates = crates.join(""),
                cargo_ebuild_ver = env!("CARGO_PKG_VERSION"),
                )
        .map_err(|err| {
            CliError::new(&format!("unable to write ebuild to disk: {}", err.description()),
                          1)
        }));

    println!("Wrote: {}", ebuild_path.display());


    Ok(None)
}
