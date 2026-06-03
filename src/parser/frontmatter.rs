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

    /// Generate table of contents
    pub toc: Option<bool>,

    /// Generate cover page
    pub cover: Option<bool>,

    /// Font family
    pub font: Option<String>,

    /// Font size in points
    #[serde(rename = "font-size")]
    pub font_size: Option<u32>,

    /// Paper format (A4, Letter)
    pub format: Option<String>,

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
    let closing_pos = after_first_delimiter
        .find("\n---")
        .or_else(|| after_first_delimiter.find("\r\n---"));

    match closing_pos {
        Some(pos) => {
            let yaml_content = &after_first_delimiter[..pos].trim();
            let remaining_content = &after_first_delimiter[pos + 4..]; // Skip past "\n---"

            // Skip any trailing newlines after the closing delimiter
            let remaining_content = remaining_content.trim_start_matches(['\n', '\r']);

            let metadata: DocumentMetadata = serde_yaml::from_str(yaml_content)
                .with_context(|| "Failed to parse YAML frontmatter")?;

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
        let (metadata, remaining) = extract_frontmatter(content).unwrap();

        // Should treat as no frontmatter
        assert!(metadata.title.is_none());
    }
}
