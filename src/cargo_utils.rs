use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::Method;
use cargo::core::{Package, PackageSet, Resolve, Workspace};
use cargo::ops;
use cargo::util::{important_paths, CargoResult};
use cargo::Config;

/// Finds the root Cargo.toml of the workspace
pub fn workspace(config: &Config) -> CargoResult<Workspace> {
    let root = important_paths::find_root_manifest_for_wd(&config.cwd())?;
    Workspace::new(&root, config)
}

/// Generates a package registry by using the Cargo.lock or creating one as necessary
pub fn registry<'a>(config: &'a Config, package: &Package) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources(&[package.package_id().source_id().clone()])?;
    Ok(registry)
}

/// Resolve the packages necessary for the workspace
pub fn resolve<'a>(
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
