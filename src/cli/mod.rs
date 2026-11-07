//! Command-line interface module

mod args;

pub use args::Args;

use anyhow::{Context, Result};
use std::path::Path;

/// Read the input markdown file and return its contents
pub fn read_input_file(path: &Path) -> Result<String> {
	std::fs::read_to_string(path).with_context(|| format!("Failed to read input file: {}", path.display()))
}
