use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::Method;
use cargo::core::{Package, PackageSet, Resolve, Workspace};
use cargo::ops;
use cargo::util::{important_paths, CargoResult};
use cargo::Config;
use std::path::PathBuf;

/// Represents the package we are trying to generate a recipe for
pub struct PackageInfo<'cfg> {
    cfg: &'cfg Config,
    current_manifest: PathBuf,
    ws: Workspace<'cfg>,
}

impl<'cfg> PackageInfo<'cfg> {
    /// creates our package info from the config and the manifest_path,
    /// which may not be provided
    pub fn new(config: &Config) -> CargoResult<PackageInfo> {
        let root = important_paths::find_root_manifest_for_wd(&config.cwd())?;
        let ws = Workspace::new(&root, config).unwrap();
        Ok(PackageInfo {
            cfg: config,
            current_manifest: root,
            ws,
        })
    }

    /// provides the current package we are working with
    pub fn package(&self) -> CargoResult<&Package> {
        self.ws.current()
    }

    /// Generates a package registry by using the Cargo.lock or
    /// creating one as necessary
    pub fn registry(&self) -> CargoResult<PackageRegistry<'cfg>> {
        let mut registry = PackageRegistry::new(self.cfg)?;
        let package = self.package()?;
        registry.add_sources(&[package.package_id().source_id().clone()])?;
        Ok(registry)
    }

    /// Resolve the packages necessary for the workspace
    pub fn resolve(&self) -> CargoResult<(PackageSet<'cfg>, Resolve)> {
        // build up our registry
        let mut registry = self.registry()?;

        // resolve our dependencies
        let (packages, resolve) = ops::resolve_ws(&self.ws)?;

        // resolve with all features set so we ensure we get all of the depends downloaded
        let resolve = ops::resolve_with_previous(
            &mut registry,
            &self.ws,
            Method::Everything, // resolve it all
            Some(&resolve),     // previous
            None,               // don't avoid any
            &[],                // specs
            true,               // register patches
            false,              // warn
        )?;

        Ok((packages, resolve))
    }

    /// packages that are part of a workspace are a sub directory from the
    /// top level which we need to record, this provides us with that
    /// relative directory
    pub fn rel_dir(&self) -> CargoResult<PathBuf> {
        // this is the top level of the workspace
        let root = self.ws.root().to_path_buf();
        // path where our current package's Cargo.toml lives
        let cwd = self.current_manifest.parent().unwrap();

        Ok(cwd.strip_prefix(&root).map(|p| p.to_path_buf()).unwrap())
    }
}
