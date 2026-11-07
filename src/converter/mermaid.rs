//! Mermaid diagram support using kroki.io
//!
//! Renders Mermaid code blocks as SVG images via the kroki.io API.
//! SVGs are post-processed to convert HTML foreignObject elements to native
//! SVG text for compatibility with Typst's SVG renderer.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

use anyhow::{Context, Result};
use tracing::debug;

/// Check if a code block language indicates Mermaid
pub fn is_mermaid_block(lang: &str) -> bool {
	matches!(lang.to_lowercase().as_str(), "mermaid" | "mmd" | "mermaid-js")
}

/// Mermaid renderer using kroki.io API
pub struct KrokiRenderer {
	/// HTTP agent with TLS
	agent: ureq::Agent,
	/// Cache directory for SVG files
	cache_dir: PathBuf,
	/// Cache of source hash -> SVG file path
	cache: RwLock<HashMap<String, PathBuf>>,
	/// Counter for generating unique filenames
	counter: RwLock<usize>,
}

impl KrokiRenderer {
	/// Create a new kroki renderer
	pub fn new(cache_dir: PathBuf) -> Result<Self> {
		fs::create_dir_all(&cache_dir).with_context(|| format!("Failed to create mermaid cache dir: {:?}", cache_dir))?;

		let agent = ureq::Agent::config_builder()
			.tls_config(ureq::tls::TlsConfig::builder().provider(ureq::tls::TlsProvider::NativeTls).build())
			.build()
			.new_agent();

		Ok(Self { agent, cache_dir, cache: RwLock::new(HashMap::new()), counter: RwLock::new(0) })
	}

	/// Render a mermaid diagram and return the path to the SVG file
	pub fn render(&self, source: &str) -> Result<PathBuf> {
		// Create a simple hash of the source for caching
		let hash = simple_hash(source);

		// Check cache
		if let Ok(cache) = self.cache.read()
			&& let Some(path) = cache.get(&hash)
			&& path.exists()
		{
			debug!("Using cached mermaid SVG: {:?}", path);
			return Ok(path.clone());
		}

		// Generate unique filename
		let filename = {
			let mut counter = self.counter.write().unwrap();
			*counter += 1;
			format!("mermaid-{}.svg", *counter)
		};
		let svg_path = self.cache_dir.join(&filename);

		// Fetch from kroki.io
		let url = get_kroki_url(source);
		debug!("Fetching mermaid diagram from kroki.io");
		debug!("URL: {}", url);

		let response = self.agent.get(&url).call().with_context(|| format!("Failed to fetch mermaid diagram from {}", url))?;

		if response.status() != 200 {
			anyhow::bail!("kroki.io returned HTTP {}", response.status());
		}

		// Read the SVG content
		let mut svg_content = String::new();
		response.into_body().into_reader().read_to_string(&mut svg_content).context("Failed to read SVG response")?;

		// Post-process SVG to convert foreignObject/HTML to native SVG text
		// This is necessary because Typst's SVG renderer doesn't support foreignObject
		let svg_content = convert_foreign_object_to_text(&svg_content);

		// Save to file
		fs::write(&svg_path, &svg_content).with_context(|| format!("Failed to write SVG to {:?}", svg_path))?;

		debug!("Saved mermaid SVG to {:?}", svg_path);

		// Update cache
		if let Ok(mut cache) = self.cache.write() {
			cache.insert(hash, svg_path.clone());
		}

		Ok(svg_path)
	}
}

/// Encode diagram source for kroki.io URL
/// Uses zlib compression and base64url encoding
fn encode_kroki(source: &str) -> String {
	use flate2::Compression;
	use flate2::write::ZlibEncoder;
	use std::io::Write;

	// Compress with zlib (includes header/trailer, required by kroki)
	let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
	encoder.write_all(source.as_bytes()).unwrap();
	let compressed = encoder.finish().unwrap();

	// Encode as base64url (no padding)
	base64_url_encode(&compressed)
}

/// Base64url encode without padding
fn base64_url_encode(data: &[u8]) -> String {
	const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

	let mut result = String::new();
	let mut bits = 0u32;
	let mut bit_count = 0;

	for &byte in data {
		bits = (bits << 8) | u32::from(byte);
		bit_count += 8;

		while bit_count >= 6 {
			bit_count -= 6;
			let index = ((bits >> bit_count) & 0x3F) as usize;
			result.push(ALPHABET[index] as char);
		}
	}

	if bit_count > 0 {
		let index = ((bits << (6 - bit_count)) & 0x3F) as usize;
		result.push(ALPHABET[index] as char);
	}

	result
}

/// Get the kroki.io URL for a mermaid diagram
fn get_kroki_url(source: &str) -> String {
	let encoded = encode_kroki(source);
	format!("https://kroki.io/mermaid/svg/{}", encoded)
}

/// Simple string hash for caching
fn simple_hash(s: &str) -> String {
	let mut hash: u64 = 0;
	for byte in s.bytes() {
		hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
	}
	format!("{:016x}", hash)
}

/// Convert foreignObject elements with HTML content to native SVG text elements
/// This is necessary because Typst's SVG renderer doesn't support foreignObject
fn convert_foreign_object_to_text(svg: &str) -> String {
	// Strategy: Find each foreignObject element and its matching closing tag,
	// then extract content and replace with SVG text.
	// This avoids the .*? greedy matching issues.

	let mut result = String::new();
	let mut pos = 0;

	while pos < svg.len() {
		// Find next foreignObject
		if let Some(fo_start) = svg[pos..].find("<foreignObject") {
			let fo_start = pos + fo_start;

			// Copy everything before this foreignObject
			result.push_str(&svg[pos..fo_start]);

			// Find the end of the opening tag
			if let Some(tag_end) = svg[fo_start..].find('>') {
				let tag_end = fo_start + tag_end;
				let opening_tag = &svg[fo_start..=tag_end];

				// Extract width and height from opening tag
				let width = extract_attr(opening_tag, "width").unwrap_or(100.0);
				let height = extract_attr(opening_tag, "height").unwrap_or(24.0);

				// Find closing </foreignObject>
				if let Some(close_start) = svg[tag_end..].find("</foreignObject>") {
					let close_start = tag_end + close_start;
					let close_end = close_start + "</foreignObject>".len();

					// Get the inner content
					let inner = &svg[tag_end + 1..close_start];

					// Extract text from the inner HTML
					let text = extract_text_from_html(inner);

					if text.is_empty() {
						// Empty foreignObject - skip it entirely
					} else {
						// Create SVG text element centered within the foreignObject bounds
						result.push_str(&format!(
							r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="central" style="font-family: sans-serif; font-size: 14px; fill: #333;">{}</text>"#,
							width / 2.0,
							height / 2.0,
							escape_svg_text(&text)
						));
					}

					pos = close_end;
					continue;
				}
			}
		}

		// No more foreignObjects - copy the rest
		result.push_str(&svg[pos..]);
		break;
	}

	result
}

/// Extract a numeric attribute value from an XML tag
fn extract_attr(tag: &str, attr: &str) -> Option<f64> {
	let pattern = format!(r#"{}="([^"]+)""#, attr);
	let re = regex::Regex::new(&pattern).ok()?;
	re.captures(tag).and_then(|caps| caps.get(1)).and_then(|m| m.as_str().parse().ok())
}

/// Extract plain text from mermaid HTML content
/// Handles both nodeLabel (<p>text</p>) and edgeLabel patterns
fn extract_text_from_html(html: &str) -> String {
	// Try to find text in <p>...</p> tags (node labels)
	if let Some(p_start) = html.find("<p>")
		&& let Some(p_end) = html[p_start..].find("</p>")
	{
		let text = &html[p_start + 3..p_start + p_end];
		return html_decode(text.trim());
	}

	// Try to find text in span.edgeLabel (edge labels with text)
	// Look for text between > and <  Pattern: <span class="edgeLabel">TEXT</span>
	if html.contains("edgeLabel")
		&& let Some(span_end) = html.find("edgeLabel")
		&& let Some(gt) = html[span_end..].find('>')
	{
		let text_start = span_end + gt + 1;
		if let Some(lt) = html[text_start..].find('<') {
			let text = &html[text_start..text_start + lt];
			return html_decode(text.trim());
		}
	}

	String::new()
}

/// Decode HTML entities
fn html_decode(s: &str) -> String {
	s.replace("&amp;", "&")
		.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("&quot;", "\"")
		.replace("&#39;", "'")
		.replace("&apos;", "'")
}

/// Escape text for SVG
fn escape_svg_text(s: &str) -> String {
	s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Generate Typst code to display the rendered mermaid SVG
pub fn generate_image_typst(svg_relative_path: &str) -> String {
	format!(
		r#"#align(center)[
  #image("{}")
]

"#,
		svg_relative_path.replace('\\', "/")
	)
}

/// Generate fallback Typst code when kroki.io rendering fails
pub fn generate_fallback_typst(source: &str, error: &str) -> String {
	let escaped = source.replace('`', "'").trim().to_string();

	format!(
		r##"#block(
  fill: rgb("#fff3cd"),
  stroke: 1pt + rgb("#ffc107"),
  inset: 1em,
  radius: 4pt,
  width: 100%,
)[
  #text(weight: "bold")[Mermaid Diagram]
  #parbreak()
  #text(size: 0.9em, fill: rgb("#856404"))[Could not render: {}]
  #parbreak()
```
{}
```
]

"##,
		error.replace('"', "'"),
		escaped
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_mermaid_block() {
		assert!(is_mermaid_block("mermaid"));
		assert!(is_mermaid_block("Mermaid"));
		assert!(is_mermaid_block("mmd"));
		assert!(!is_mermaid_block("rust"));
	}

	#[test]
	fn test_encode_kroki() {
		// Just verify it produces non-empty output
		let encoded = encode_kroki("graph LR\n    A --> B");
		assert!(!encoded.is_empty());
		assert!(encoded.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
	}

	#[test]
	fn test_html_decode() {
		assert_eq!(html_decode("A &amp; B"), "A & B");
		assert_eq!(html_decode("&lt;tag&gt;"), "<tag>");
	}
}
