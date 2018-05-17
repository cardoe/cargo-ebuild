extern crate cargo;
extern crate time;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

pub mod ebuild;
pub mod cargo_utils;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo ebuild")]
pub struct Cli {
    #[structopt(subcommand)] // the real cargo-ebuild commands
    cmd: Command,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u32,

    /// No output printed to stdout
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "build")]
    /// Build an ebuild file from a cargo project
    Build {
        #[structopt(long = "unstable-flags", short = "Z")]
        unstable_flags: Vec<String>,
    },
}

use cargo::Config;
use cargo_utils::*;
use cargo::util::CargoError;
use ebuild::*;
use failure::Error;
use failure::err_msg;

/// Quite huge all-in-one func that generates and Ebuild from some cli
/// cli configuration
pub fn run_cargo_ebuild(cli: Cli) -> Result<Ebuild, Error> {
    // Here will be the match of the commands, now just example
    let flags = match cli.cmd {
        Command::Build {
            unstable_flags,
        } => unstable_flags
    };

    // build the crate URIs
    let mut config = Config::default()?;

    config
        .configure(
            cli.verbose,
            Some(cli.quiet),
            &None, // color
            false, // frozen
            false, // locked
            &flags
        )?;
        // .map_err(|e| e.to_string())?;

    // Build up data about the package we are attempting to generate a recipe for
    let md = PackageInfo::new(&config)?;

    // Our current package
    let package = md.package()?;

    let _crate_root = package
        .manifest_path()
        .parent()
        .expect("Cargo.toml must have a parent");

    // Resolve all dependencies (generate or use Cargo.lock as necessary)
    let resolve = md.resolve()?;

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
                    .ok_or(format_err!("No package.repository set in your Cargo.toml"))
            },
            |s| Ok(s),
        )?
        .trim();

    // compute the relative directory into the repo our Cargo.toml is at
    let _rel_dir = md.rel_dir()?;

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
