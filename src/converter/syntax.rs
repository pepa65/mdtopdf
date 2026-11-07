//! Syntax highlighting for code blocks using syntect
//!
//! This module provides syntax highlighting functionality, though Typst
//! has built-in highlighting for many languages. We use this for:
//! - Enhanced styling (code block titles, custom themes)
//! - Fallback for languages Typst doesn't support

/// Code block information extracted from markdown
#[derive(Debug, Clone)]
pub struct CodeBlockInfo {
	/// The programming language (e.g., "rust", "python")
	pub language: Option<String>,
	/// Optional title for the code block
	pub title: Option<String>,
	/// The code content
	pub content: String,
}

impl CodeBlockInfo {
	/// Parse code block info from the fence info string and content
	///
	/// Supports formats like:
	/// - `rust`
	/// - `rust title="My Code"`
	/// - `python # title: Example`
	pub fn parse(info: &str, content: &str) -> Self {
		let mut language = None;
		let mut title = None;

		// Parse the info string (e.g., "rust title=\"My Code\"")
		let parts: Vec<&str> = info.split_whitespace().collect();
		if let Some(first) = parts.first()
			&& !first.is_empty()
		{
			language = Some(first.to_string());
		}

		// Look for title="..." or title='...'
		for part in &parts[1..] {
			if let Some(rest) = part.strip_prefix("title=") {
				let t = rest.trim_matches(|c| c == '"' || c == '\'');
				title = Some(t.to_string());
			}
		}

		// Also check first line of content for `# title: ...` pattern
		if title.is_none()
			&& let Some(first_line) = content.lines().next()
		{
			let trimmed = first_line.trim();
			if let Some(rest) = trimmed.strip_prefix("# title:") {
				title = Some(rest.trim().to_string());
			} else if let Some(rest) = trimmed.strip_prefix("// title:") {
				title = Some(rest.trim().to_string());
			} else if let Some(rest) = trimmed.strip_prefix("-- title:") {
				title = Some(rest.trim().to_string());
			}
		}

		Self { language, title, content: content.to_string() }
	}

	/// Get the content, optionally stripping the title comment line
	pub fn content_without_title(&self) -> &str {
		if self.title.is_some() {
			// Check if first line is a title comment
			if let Some(first_line) = self.content.lines().next() {
				let trimmed = first_line.trim();
				if trimmed.starts_with("# title:") || trimmed.starts_with("// title:") || trimmed.starts_with("-- title:") {
					// Return content after first line
					if let Some(pos) = self.content.find('\n') {
						return &self.content[pos + 1..];
					}
				}
			}
		}
		&self.content
	}
}

/// Check if a language is supported by Typst's built-in highlighting
#[cfg(test)]
pub fn is_typst_supported_language(lang: &str) -> bool {
	// Typst supports these languages natively
	matches!(
		lang.to_lowercase().as_str(),
		"rust"
			| "python"
			| "py" | "javascript"
			| "js" | "typescript"
			| "ts" | "java"
			| "c" | "cpp"
			| "c++" | "csharp"
			| "cs" | "go"
			| "ruby" | "rb"
			| "php" | "swift"
			| "kotlin"
			| "scala" | "r"
			| "sql" | "html"
			| "css" | "xml"
			| "json" | "yaml"
			| "yml" | "toml"
			| "markdown"
			| "md" | "bash"
			| "sh" | "shell"
			| "powershell"
			| "ps1" | "dockerfile"
			| "makefile"
			| "latex" | "tex"
			| "typst" | "typ"
	)
}

/// Generate Typst code for a styled code block
pub fn generate_code_block_typst(info: &CodeBlockInfo) -> String {
	let content = info.content_without_title();
	let escaped = content.replace('`', "\\`").trim_end().to_string();

	let lang_str = info.language.as_deref().unwrap_or("");

	if let Some(ref title) = info.title {
		// Code block with title
		format!(
			r##"#block(
  fill: rgb("#f6f8fa"),
  stroke: 1pt + rgb("#e1e4e8"),
  inset: 0pt,
  radius: 4pt,
  width: 100%,
)[
  #block(
    fill: rgb("#e1e4e8"),
    inset: (x: 1em, y: 0.5em),
    width: 100%,
    radius: (top: 4pt),
  )[#text(size: 0.9em, fill: rgb("#57606a"))[{}]]
  #block(inset: 1em)[
```{}
{}
```
  ]
]

"##,
			title, lang_str, escaped
		)
	} else {
		// Simple code block
		format!(
			r##"#block(
  fill: rgb("#f6f8fa"),
  stroke: 1pt + rgb("#e1e4e8"),
  inset: 1em,
  radius: 4pt,
  width: 100%,
)[
```{}
{}
```
]

"##,
			lang_str, escaped
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_simple_language() {
		let info = CodeBlockInfo::parse("rust", "fn main() {}");
		assert_eq!(info.language, Some("rust".to_string()));
		assert_eq!(info.title, None);
	}

	#[test]
	fn test_parse_language_with_title_attr() {
		let info = CodeBlockInfo::parse("python title=\"Example\"", "print('hello')");
		assert_eq!(info.language, Some("python".to_string()));
		assert_eq!(info.title, Some("Example".to_string()));
	}

	#[test]
	fn test_parse_title_from_comment() {
		let info = CodeBlockInfo::parse("rust", "// title: My Example\nfn main() {}");
		assert_eq!(info.language, Some("rust".to_string()));
		assert_eq!(info.title, Some("My Example".to_string()));
	}

	#[test]
	fn test_content_without_title() {
		let info = CodeBlockInfo::parse("rust", "// title: Test\nfn main() {}");
		assert_eq!(info.content_without_title(), "fn main() {}");
	}

	#[test]
	fn test_typst_supported_languages() {
		assert!(is_typst_supported_language("rust"));
		assert!(is_typst_supported_language("Python"));
		assert!(is_typst_supported_language("JS"));
		assert!(!is_typst_supported_language("brainfuck"));
	}
}
