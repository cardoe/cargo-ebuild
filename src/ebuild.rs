//!
//! Ebuild management
//!
//! This module provides some very basic abstraction for getting the hands
//! dirty on gentoo's ebuild.
//!
//!

use std::io;

/// Hold all the usefull configurion of an ebuild
#[derive(Debug, Default)]
pub struct Ebuild {
    name: String,
    description: String,
    homepage: String,
    license: String,
    crates: String,
    version: String,
    provider: String,
    provider_version: String,
    this_year: i32,
}

impl Ebuild {
    /// Ebuild information
    pub fn new(
        name: &str,
        description: &str,
        homepage: &str,
        license: &str,
        crates: &str,
        version: &str,
        provider: &str,
        provider_version: &str,
        year: i32,
    ) -> Ebuild {
        Ebuild {
            name: name.to_string(),
            description: description.to_string(),
            homepage: homepage.to_string(),
            license: license.to_string(),
            crates: crates.to_string(),
            version: version.to_string(),
            provider: provider.to_string(),
            provider_version: provider_version.to_string(),
            this_year: year,
        }
    }

    /// Get ebuild's name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get ebuild's version
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Write the ebuild from the template to a buffer
    pub fn write(&self, file: &mut io::Write) -> io::Result<()> {
        write!(
            file,
            include_str!("ebuild.template"),
            description = self.description,
            homepage = self.homepage,
            license = self.license,
            crates = self.crates,
            provider = self.provider,
            provider_version = self.provider_version,
            this_year = self.this_year
        )
    }
}
