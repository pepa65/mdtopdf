//! Typst package downloading and caching
//!
//! Downloads packages from the Typst package registry and caches them locally.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use flate2::read::GzDecoder;
use tar::Archive;
use tracing::debug;

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
	/// Parse a Typst package specification such as @preview/oxdraw:0.1.0
	#[cfg(test)]
	pub fn parse(input: &str) -> Option<Self> {
		let input = input.strip_prefix('@')?;
		let (namespace, rest) = input.split_once('/')?;
		let (name, version) = rest.split_once(':')?;

		if namespace.is_empty() || name.is_empty() || version.is_empty() {
			return None;
		}

		Some(Self { namespace: namespace.to_string(), name: name.to_string(), version: version.to_string() })
	}

	/// Get the download URL for this package
	pub fn download_url(&self) -> String {
		format!("{}/{}/{}-{}.tar.gz", PACKAGE_REGISTRY, self.namespace, self.name, self.version)
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
		Self::with_cache_dir(cache_dir)
	}

	/// Create a package manager with a custom cache directory
	pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
		fs::create_dir_all(&cache_dir).with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;

		// Create HTTP agent using native TLS
		let agent = ureq::Agent::config_builder()
			.tls_config(ureq::tls::TlsConfig::builder().provider(ureq::tls::TlsProvider::NativeTls).build())
			.build()
			.new_agent();

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
		debug!("Downloading package {} from {}", spec.name, url);

		// Download the tarball using our TLS-enabled agent
		let response = self.agent.get(&url).call().map_err(|e| match e {
			ureq::Error::StatusCode(code) => {
				anyhow!("HTTP {} downloading {}", code, url)
			}
			e => {
				anyhow!("Error downloading {}: {}", url, e)
			}
		})?;

		// Require a complete, successful download
		if response.status() != 200 {
			return Err(anyhow!("Failed to download package {}: HTTP {}", spec.name, response.status()));
		}

		// Read the response body
		let mut body = Vec::new();
		response.into_body().into_reader().read_to_end(&mut body).context("Failed to read package data")?;

		debug!("Downloaded {} bytes for package {}", body.len(), spec.name);

		// Create the destination directory
		fs::create_dir_all(dest).with_context(|| format!("Failed to create package directory: {:?}", dest))?;

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
				fs::create_dir_all(&dest_path).with_context(|| format!("Failed to create directory: {:?}", dest_path))?;
			} else {
				// Ensure parent directory exists
				if let Some(parent) = dest_path.parent() {
					fs::create_dir_all(parent)?;
				}

				let mut file = File::create(&dest_path).with_context(|| format!("Failed to create file: {:?}", dest_path))?;
				io::copy(&mut entry, &mut file).with_context(|| format!("Failed to extract file: {:?}", dest_path))?;
			}
		}

		debug!("Extracted package {} to {:?}", spec.name, dest);

		Ok(())
	}
}

/// Get the default cache directory for Typst packages
fn get_cache_dir() -> Result<PathBuf> {
	// Try to use the standard cache directory
	if let Some(cache_dir) = dirs::cache_dir() {
		return Ok(cache_dir.join("mdtopdf").join("packages"));
	}

	// Fallback to a directory in the current working directory
	Ok(PathBuf::from(".mdtopdf-cache").join("packages"))
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
		assert_eq!(spec.download_url(), "https://packages.typst.org/preview/oxdraw-0.1.0.tar.gz");
	}

	#[test]
	fn test_cache_dir_name() {
		let spec = PackageSpec::parse("@preview/oxdraw:0.1.0").unwrap();
		assert_eq!(spec.cache_dir_name(), "preview/oxdraw/0.1.0");
	}
}
