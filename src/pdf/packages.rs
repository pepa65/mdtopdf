//! Typst package downloading and caching
//!
//! Downloads packages from the Typst package registry and caches them locally.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use tar::Archive;
use tracing::{debug, info};

/// The Typst package registry URL
const PACKAGE_REGISTRY: &str = "https://packages.typst.org";

/// Package specification parsed from an import like `@preview/oxdraw:0.1.0`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageSpec {
    /// Package namespace (e.g., "preview")
    pub namespace: String,
    /// Package name (e.g., "oxdraw")
    pub name: String,
    /// Package version (e.g., "0.1.0")
    pub version: String,
}

impl PackageSpec {
    /// Parse a package specification from an import string
    /// e.g., "@preview/oxdraw:0.1.0" -> PackageSpec { namespace: "preview", name: "oxdraw", version: "0.1.0" }
    pub fn parse(spec: &str) -> Option<PackageSpec> {
        let spec = spec.trim();

        // Must start with @
        if !spec.starts_with('@') {
            return None;
        }

        let rest = &spec[1..];

        // Split namespace/name:version
        let (namespace_name, version) = rest.split_once(':')?;
        let (namespace, name) = namespace_name.split_once('/')?;

        Some(PackageSpec {
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: version.to_string(),
        })
    }

    /// Get the download URL for this package
    pub fn download_url(&self) -> String {
        format!(
            "{}/{}/{}-{}.tar.gz",
            PACKAGE_REGISTRY, self.namespace, self.name, self.version
        )
    }

    /// Get the directory name for this package in the cache
    pub fn cache_dir_name(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.name, self.version)
    }
}

/// Package manager for downloading and caching Typst packages
pub struct PackageManager {
    /// Root directory for cached packages
    cache_dir: PathBuf,
    /// HTTP agent with TLS configured
    agent: ureq::Agent,
}

impl PackageManager {
    /// Create a new package manager with the default cache directory
    pub fn new() -> Result<Self> {
        let cache_dir = get_cache_dir()?;
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;

        // Create TLS connector using native-tls
        let tls_connector = native_tls::TlsConnector::new()
            .context("Failed to create TLS connector")?;

        // Create HTTP agent with TLS
        let agent = ureq::AgentBuilder::new()
            .tls_connector(Arc::new(tls_connector))
            .build();

        Ok(Self { cache_dir, agent })
    }

    /// Create a package manager with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;

        // Create TLS connector using native-tls
        let tls_connector = native_tls::TlsConnector::new()
            .context("Failed to create TLS connector")?;

        // Create HTTP agent with TLS
        let agent = ureq::AgentBuilder::new()
            .tls_connector(Arc::new(tls_connector))
            .build();

        Ok(Self { cache_dir, agent })
    }

    /// Get the path to a package, downloading it if necessary
    pub fn get_package(&self, spec: &PackageSpec) -> Result<PathBuf> {
        let package_dir = self.cache_dir.join(spec.cache_dir_name());

        // Check if already cached
        if package_dir.exists() && package_dir.join("typst.toml").exists() {
            debug!("Package {} already cached at {:?}", spec.name, package_dir);
            return Ok(package_dir);
        }

        // Download and extract
        self.download_package(spec, &package_dir)?;

        Ok(package_dir)
    }

    /// Download and extract a package
    fn download_package(&self, spec: &PackageSpec, dest: &PathBuf) -> Result<()> {
        let url = spec.download_url();
        info!("Downloading package {} from {}", spec.name, url);

        // Download the tarball using our TLS-enabled agent
        let response = self.agent.get(&url)
            .call()
            .map_err(|e| {
                match e {
                    ureq::Error::Status(code, response) => {
                        anyhow!("HTTP {} downloading {}: {:?}", code, url, response.status_text())
                    }
                    ureq::Error::Transport(transport) => {
                        anyhow!("Transport error downloading {}: {}", url, transport)
                    }
                }
            })?;

        if response.status() != 200 {
            return Err(anyhow!(
                "Failed to download package {}: HTTP {}",
                spec.name,
                response.status()
            ));
        }

        // Read the response body
        let mut body = Vec::new();
        response.into_reader().read_to_end(&mut body)
            .context("Failed to read package data")?;

        info!("Downloaded {} bytes for package {}", body.len(), spec.name);

        // Create the destination directory
        fs::create_dir_all(dest)
            .with_context(|| format!("Failed to create package directory: {:?}", dest))?;

        // Extract the tarball
        let decoder = GzDecoder::new(&body[..]);
        let mut archive = Archive::new(decoder);

        // Extract all files
        for entry in archive.entries().context("Failed to read archive entries")? {
            let mut entry = entry.context("Failed to read archive entry")?;
            let path = entry.path().context("Failed to get entry path")?;
            let path = path.to_path_buf();

            debug!("Archive entry: {:?}", path);

            // Skip the "." directory entry
            if path.as_os_str() == "." {
                continue;
            }

            // Use path as-is (Typst packages don't have a root directory prefix)
            let dest_path = dest.join(&path);
            debug!("Extracting to: {:?}", dest_path);

            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&dest_path)
                    .with_context(|| format!("Failed to create directory: {:?}", dest_path))?;
            } else {
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut file = File::create(&dest_path)
                    .with_context(|| format!("Failed to create file: {:?}", dest_path))?;
                io::copy(&mut entry, &mut file)
                    .with_context(|| format!("Failed to extract file: {:?}", dest_path))?;
            }
        }

        info!("Extracted package {} to {:?}", spec.name, dest);

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

/// Get the default cache directory for Typst packages
fn get_cache_dir() -> Result<PathBuf> {
    // Try to use the standard cache directory
    if let Some(cache_dir) = dirs::cache_dir() {
        return Ok(cache_dir.join("md2pdf").join("packages"));
    }

    // Fallback to a directory in the current working directory
    Ok(PathBuf::from(".md2pdf-cache").join("packages"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        let spec = PackageSpec::parse("@preview/oxdraw:0.1.0").unwrap();
        assert_eq!(spec.namespace, "preview");
        assert_eq!(spec.name, "oxdraw");
        assert_eq!(spec.version, "0.1.0");
    }

    #[test]
    fn test_parse_package_spec_invalid() {
        assert!(PackageSpec::parse("preview/oxdraw:0.1.0").is_none()); // Missing @
        assert!(PackageSpec::parse("@preview:0.1.0").is_none()); // Missing /name
        assert!(PackageSpec::parse("@preview/oxdraw").is_none()); // Missing :version
    }

    #[test]
    fn test_download_url() {
        let spec = PackageSpec::parse("@preview/oxdraw:0.1.0").unwrap();
        assert_eq!(
            spec.download_url(),
            "https://packages.typst.org/preview/oxdraw-0.1.0.tar.gz"
        );
    }

    #[test]
    fn test_cache_dir_name() {
        let spec = PackageSpec::parse("@preview/oxdraw:0.1.0").unwrap();
        assert_eq!(spec.cache_dir_name(), "preview/oxdraw/0.1.0");
    }
}
