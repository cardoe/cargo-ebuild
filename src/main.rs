extern crate cargo;
extern crate rustc_serialize;
extern crate time;

use cargo::{Config, CliError, CliResult};
use cargo::core::{Package, Resolve};
use cargo::core::registry::PackageRegistry;
use cargo::ops;
use cargo::util::{important_paths, CargoResult};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Finds the root Cargo.toml of the workspace
fn workspace(config: &Config, manifest_path: Option<String>) -> CargoResult<Package> {
    let root = important_paths::find_root_manifest_for_wd(manifest_path, config.cwd())?;
    Package::for_path(&root, config)
}

/// Generates a package registry by using the Cargo.lock or creating one as necessary
fn registry<'a>(config: &'a Config, package: &Package) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config);
    registry.add_sources(&[package.package_id().source_id().clone()])?;
    Ok(registry)
}

/// Resolve the packages necessary for the workspace
fn resolve<'a>(registry: &mut PackageRegistry,
               package: &'a Package,
               config: &'a Config) -> CargoResult<Resolve> {
    ops::resolve_pkg(registry, package, config)
}

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

    // Load the workspace and current package
    let package = workspace(config, None)?;

    // Resolve all dependencies (generate or use Cargo.lock as necessary)
    let mut registry = registry(config, &package)?;
    let resolve = resolve(&mut registry, &package, config)?;

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
                                         CliError::new(&format!("failed to create ebuild: {}",
                                                                err.description()),
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
                this_year = 1900 + time::now().tm_year,
                )
                 .map_err(|err| {
                              CliError::new(&format!("unable to write ebuild to disk: {}",
                                                     err.description()),
                                            1)
                          }));

    println!("Wrote: {}", ebuild_path.display());


    Ok(None)
}
