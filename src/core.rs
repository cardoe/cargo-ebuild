
use cargo::Config;
use cargo::util::errors::CliError;
use cargo_utils::*;
use time;

#[derive(Debug)]
pub struct Ebuild {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub crates: String,
    pub version: String,
    pub year: i32
}

pub fn ebuild_from_cargo(options: super::Opt, config: &mut Config) -> Result<Ebuild, CliError> {
    config.configure(
        options.verbose,
        Some(options.quiet),
        // color
        &None,
        // frozen
        false,
        // locked
        false,
        // unstable flag
        &options.unstable_flags,
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
    let desc = metadata.description.as_ref().cloned().unwrap();
    // TODO: now package.name is InternedString
    // .unwrap_or_else(|| String::from(package.name()));

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

    // write the contents out
    Ok( Ebuild {
        name: package.name().to_string(),
        description: desc.trim().to_string(),
        homepage: homepage.trim().to_string(),
        license: license.trim().to_string(),
        crates: crates.join("").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        year: 1900 + time::now().tm_year,
    })
}
