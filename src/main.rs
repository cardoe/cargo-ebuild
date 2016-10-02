extern crate cargo;
extern crate rustache;
extern crate rustc_serialize;

use cargo::{Config, CliError, CliResult};
use cargo::core::Package;
use cargo::core::registry::PackageRegistry;
use cargo::ops;
use cargo::util::important_paths;
use rustache::HashBuilder;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
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
    let mut crates = Vec::<String>::new();
    for pkg in resolve.iter() {
        crates.push(format!("{}-{}\n", pkg.name(), pkg.version()));
    }

    // root package metadata
    let metadata = package.manifest().metadata();

    // package description
    let desc = metadata.description
        .as_ref()
        .map(|d| d.clone())
        .unwrap_or(String::from(package.name()));

    // package homepage
    let homepage = metadata.homepage
        .as_ref()
        .map(|h| h.clone())
        .unwrap_or(metadata.repository
            .as_ref()
            .map(|h| h.clone())
            .unwrap_or(String::from("")));

    // build up the ebuild path
    let ebuild_path = PathBuf::from(format!("{}-{}.ebuild", package.name(), package.version()));

    // build up the varibles for the template
    let data = HashBuilder::new()
        .insert_string("description", desc.trim())
        .insert_string("homepage", homepage.trim())
        .insert_string("crates", crates.join(""));

    // load the ebuild template
    let template = include_str!("ebuild.template");

    // generate the ebuild using Rustache to process the template
    let mut templ = try!(rustache::render_text(template, data)
        .map_err(|_| CliError::new("unable to generate ebuild: {}", 1)));

    // Open the file where we'll write the ebuild
    let mut file = try!(OpenOptions::new()
        .write(true)
        .create(true)
        .open(&ebuild_path)
        .map_err(|err| {
            CliError::new(&format!("failed to create ebuild: {}", err.description()),
                          1)
        }));

    // write the contents out
    try!(io::copy(&mut templ, &mut file).map_err(|err| {
        CliError::new(&format!("unable to write ebuild to disk: {}", err.description()),
                      1)
    }));

    println!("Wrote: {}", ebuild_path.display());


    Ok(None)
}
