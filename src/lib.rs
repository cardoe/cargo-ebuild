extern crate cargo;
extern crate time;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate log;

mod cargo_utils;
pub mod ebuild;

use cargo::Config;
use cargo_utils::*;
use ebuild::*;

/// This is autogenerated for the cli interface, should be in the main.rs
/// but for now is easier to keep here.
#[derive(StructOpt, Debug)]
#[structopt(name = "cargo ebuild")]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u32,

    /// No output printed to stdout
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Arguments passed to cargo
    #[structopt(short = "f", long = "unstable-flags", parse(from_str))]
    pub unstable_flags: Vec<String>,
}

/// Quite huge all-in-one func that generates and Ebuild from some cli
/// cli configuration
pub fn ebuild_from_cargo(options: Opt) -> Result<Ebuild, String> {
    // build the crate URIs
    let mut config = Config::default()
        .map_err(|e| e.to_string())?;

    config.configure(
        options.verbose,
        Some(options.quiet),
        &None, // color
        false, // frozen
        false, // locked
        &options.unstable_flags,
    ).map_err(|e| e.to_string())?;


    // Build up data about the package we are attempting to generate a recipe for
    let md = PackageInfo::new(&config)
        .map_err(|e| e.to_string())?;

    // Our current package
    let package = md.package()
        .map_err(|e| e.to_string())?;

    let _crate_root = package
        .manifest_path()
        .parent()
        .expect("Cargo.toml must have a parent");

    // Resolve all dependencies (generate or use Cargo.lock as necessary)
    let resolve = md.resolve()
        .map_err(|e| e.to_string())?;

    // build the crates the package needs
    let mut crates = resolve
        .1
        .iter()
        .filter_map(|pkg| {
            // get the source info for this package
            let src_id = pkg.source_id();
            if pkg.name() == package.name() {
                // None
                // Should be none in future
                Some(format!("{}-{}\n", pkg.name(), pkg.version()))
            } else if src_id.is_registry() {
                // this package appears in a crate registry
                Some(format!("{}-{}\n", pkg.name(), pkg.version()))
            } else if src_id.is_path() {
                // we don't want to spit out path based
                // entries since they're within the crate
                // we are packaging
                None
            } else {
                Some(format!("{} \\\n", src_id.url().to_string()))
            }
        })
        .collect::<Vec<String>>();

    // sort the crates
    crates.sort();


    // root package metadata
    let metadata = package.manifest().metadata();

    // package description is used as BitBake summary
    let summary = metadata.description.as_ref().map_or_else(
        || {
            debug!("No package.description set in your Cargo.toml, using package.name");
            package.name().to_string()
        },
        |s| s.trim().to_string(),
    );

    // package homepage (or source code location)
    let homepage = metadata
        .homepage
        .as_ref()
        .map_or_else(
            || {
                debug!("No package.homepage set in your Cargo.toml, trying package.repository");
                metadata
                    .repository
                    .as_ref()
                    .ok_or("No package.repository set in your Cargo.toml".to_owned())
            },
            |s| Ok(s),
        )?
        .trim();

    // compute the relative directory into the repo our Cargo.toml is at
    let _rel_dir = md.rel_dir()
        .map_err(|e| e.to_string())?;


    // package version
    let version = package.manifest().version().to_string();

    let license = metadata
        .license
        .as_ref()
        .cloned()
        .unwrap_or_else(|| String::from("unknown license"));

    // write the contents out
    Ok(Ebuild::new(
        &package.name().to_string(),
        summary.trim(),
        homepage.trim(),
        license.trim(),
        &crates.join(""),
        &version,
        "cargo-ebuild".as_ref(),
        env!("CARGO_PKG_VERSION"),
        1900 + time::now().tm_year,
    ))
}