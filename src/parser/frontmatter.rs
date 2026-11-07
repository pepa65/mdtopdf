//! YAML frontmatter extraction and parsing

use anyhow::{Context, Result};
use serde::Deserialize;

/// Document metadata extracted from YAML frontmatter
#[derive(Debug, Deserialize, Default, Clone)]
pub struct DocumentMetadata {
	pub title: Option<String>,
	pub subtitle: Option<String>,
	pub author: Option<String>,
	pub date: Option<String>,
	pub organization: Option<String>,

	/// Generate Table of Contents
	pub toc: Option<bool>,
	/// Table of Contents header
	#[serde(rename = "toc-header")]
	pub toc_header: Option<String>,

	/// Generate cover page
	pub cover: Option<bool>,

	/// Font family
	pub font: Option<String>,

	/// Font size in points
	#[serde(rename = "font-size")]
	pub font_size: Option<f32>,

	/// Paper format (A4, Letter)
	pub paper: Option<String>,

	/// The page margins (default: 16 mm)
	#[serde(default, deserialize_with = "deserialize_margins")]
	pub margins: Option<Margins>,

	// Header configuration
	#[serde(rename = "header-left")]
	pub header_left: Option<String>,

	#[serde(rename = "header-center")]
	pub header_center: Option<String>,

	#[serde(rename = "header-right")]
	pub header_right: Option<String>,

	// Footer configuration
	#[serde(rename = "footer-left")]
	pub footer_left: Option<String>,

	#[serde(rename = "footer-center")]
	pub footer_center: Option<String>,

	#[serde(rename = "footer-right")]
	pub footer_right: Option<String>,

	/// Show page numbers
	#[serde(rename = "page-numbers")]
	pub page_numbers: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Margins {
	pub values: [f32; 4],
	pub unit: String,
}

pub fn normalize_margins(values: &[f32]) -> Result<[f32; 4], String> {
	if !(1..=4).contains(&values.len()) {
		return Err("Margins require 1 to 4 values".into());
	}

	if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
		return Err("Margins must be finite, non-negative numbers".into());
	}

	Ok(match values {
		[a] => [*a; 4],
		[a, b] => [*a, *b, *a, *b],
		[a, b, c] => [*a, *b, *c, *b],
		[a, b, c, d] => [*a, *b, *c, *d],
		_ => unreachable!(),
	})
}

pub fn parse_margins(input: &[&str]) -> Result<Margins, String> {
	let (values, unit) = match input.last().copied() {
		Some("mm" | "cm" | "in" | "pt") => (&input[..input.len() - 1], *input.last().unwrap()),
		_ => (input, "mm"),
	};

	let numbers = values
		.iter()
		.map(|s| s.parse::<f32>().map_err(|_| format!("Invalid margin value or unit: {s}")))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(Margins { values: normalize_margins(&numbers)?, unit: unit.to_string() })
}

fn deserialize_margins<'de, D>(deserializer: D) -> Result<Option<Margins>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	use serde::de::Error;

	#[derive(serde::Deserialize)]
	#[serde(untagged)]
	enum MarginInput {
		Text(String),
		Number(f32),
		Values(Vec<serde_yaml::Value>),
	}

	let input = MarginInput::deserialize(deserializer)?;

	let parts: Vec<String> = match input {
		MarginInput::Text(s) => s.split_whitespace().map(str::to_string).collect(),
		MarginInput::Number(v) => vec![v.to_string()],
		MarginInput::Values(values) => values
			.into_iter()
			.map(|v| match v {
				serde_yaml::Value::String(s) => Ok(s),
				serde_yaml::Value::Number(n) => Ok(n.to_string()),
				_ => Err(D::Error::custom("Invalid margin value")),
			})
			.collect::<Result<_, _>>()?,
	};

	let values: Vec<&str> = parts.iter().map(String::as_str).collect();

	parse_margins(&values).map(Some).map_err(D::Error::custom)
}

/// Extract YAML frontmatter from markdown content
///
/// Frontmatter is expected to be delimited by `---` at the start of the document.
/// Returns the parsed metadata and the remaining markdown content.
pub fn extract_frontmatter(content: &str) -> Result<(DocumentMetadata, String)> {
	let content = content.trim_start();

	// Check if the document starts with frontmatter delimiter
	if !content.starts_with("---") {
		return Ok((DocumentMetadata::default(), content.to_string()));
	}

	// Find the closing delimiter
	let after_first_delimiter = &content[3..];
	let closing_pos = after_first_delimiter.find("\n---").or_else(|| after_first_delimiter.find("\r\n---"));

	match closing_pos {
		Some(pos) => {
			let yaml_content = &after_first_delimiter[..pos].trim();
			let remaining_content = &after_first_delimiter[pos + 4..]; // Skip past "\n---"
			let metadata: DocumentMetadata = serde_yaml::from_str(yaml_content).with_context(|| "Failed to parse YAML frontmatter")?;

			// Skip any trailing newlines after the closing delimiter
			let remaining_content = remaining_content.trim_start_matches(['\n', '\r']);
			Ok((metadata, remaining_content.to_string()))
		}
		None => {
			// No closing delimiter found, treat entire content as markdown
			Ok((DocumentMetadata::default(), content.to_string()))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_no_frontmatter() {
		let content = "# Hello World\n\nThis is some content.";
		let (metadata, remaining) = extract_frontmatter(content).unwrap();

		assert!(metadata.title.is_none());
		assert_eq!(remaining, content);
	}

	#[test]
	fn test_with_frontmatter() {
		let content = r#"---
title: My Document
author: John Doe
toc: true
---

# Hello World

This is some content."#;

		let (metadata, remaining) = extract_frontmatter(content).unwrap();

		assert_eq!(metadata.title, Some("My Document".to_string()));
		assert_eq!(metadata.author, Some("John Doe".to_string()));
		assert_eq!(metadata.toc, Some(true));
		assert!(remaining.starts_with("# Hello World"));
	}

	#[test]
	fn test_frontmatter_with_headers_and_footers() {
		let content = r#"---
title: Report
header-left: Company Name
header-right: Confidential
footer-center: Page
page-numbers: true
---

Content here."#;

		let (metadata, _) = extract_frontmatter(content).unwrap();

		assert_eq!(metadata.header_left, Some("Company Name".to_string()));
		assert_eq!(metadata.header_right, Some("Confidential".to_string()));
		assert_eq!(metadata.footer_center, Some("Page".to_string()));
		assert_eq!(metadata.page_numbers, Some(true));
	}

	#[test]
	fn test_unclosed_frontmatter() {
		let content = "---\ntitle: Test\n\nNo closing delimiter";
		let (metadata, _remaining) = extract_frontmatter(content).unwrap();

		// Should treat as no frontmatter
		assert!(metadata.title.is_none());
	}
}
