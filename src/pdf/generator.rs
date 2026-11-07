//! PDF generator using Typst compiler

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tracing::debug;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{DiagSpanKind, FileId, RootedPath, Source, VirtualPath, VirtualRoot, package::PackageSpec as TypstPackageSpec};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

use super::packages::{PackageManager, PackageSpec};

/// Generate PDF from Typst source content
pub fn generate_pdf(typst_content: &str, base_path: Option<&Path>) -> Result<Vec<u8>> {
	let world = Md2PdfWorld::new(typst_content, base_path)?;

	// Compile the document
	let result = typst::compile(&world);

	for warning in result.warnings {
		debug!("Typst {:?}: {}", warning.severity, warning.message);
	}

	match result.output {
		Ok(doc) => {
			// Export to PDF
			let pdf_bytes = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default()).map_err(|e| anyhow!("PDF export error: {:?}", e))?;
			Ok(pdf_bytes)
		}
		Err(errors) => {
			let error_messages: Vec<String> = errors.iter().map(|e| format_diagnostic(&world, e)).collect();
			Err(anyhow!("Typst compilation failed:\n{}", error_messages.join("\n")))
		}
	}
}

fn format_diagnostic(world: &Md2PdfWorld, diag: &SourceDiagnostic) -> String {
	let (id, range) = match diag.span.get() {
		DiagSpanKind::Detached => return diag.message.to_string(),
		DiagSpanKind::Number { id, num, sub_range } => {
			let Ok(source) = world.source(id) else {
				return diag.message.to_string();
			};
			let Some(range) = source.range(num, sub_range) else {
				return diag.message.to_string();
			};
			(id, range)
		}
		DiagSpanKind::Range { id, range } => (id, range),
	};

	if let Ok(source) = world.source(id)
		&& let Some(line) = source.lines().byte_to_line(range.start)
	{
		return format!("Line {}: {}", line + 1, diag.message);
	}

	diag.message.to_string()
}

/// Custom World implementation for mdtopdf
struct Md2PdfWorld {
	/// The main source file
	main_source: Source,
	/// The library with standard functions
	library: LazyHash<Library>,
	/// Font book with available fonts
	book: LazyHash<FontBook>,
	/// Loaded fonts
	fonts: Vec<Font>,
	/// Base path for resolving relative paths
	base_path: PathBuf,
	/// Cached files (thread-safe)
	files: RwLock<HashMap<FileId, Bytes>>,
	/// Cached sources (thread-safe)
	sources: RwLock<HashMap<FileId, Source>>,
	/// Package manager for downloading Typst packages
	package_manager: PackageManager,
}

impl Md2PdfWorld {
	fn new(source_content: &str, base_path: Option<&Path>) -> Result<Self> {
		let main_id = FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new("main.typ")?));
		let main_source = Source::new(main_id, source_content.to_string());

		let base_path = base_path.map(|p| p.to_path_buf()).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

		// Load fonts
		let (book, fonts) = load_fonts();

		// Create package manager
		let package_manager = PackageManager::new()?;

		Ok(Self {
			main_source,
			library: LazyHash::new(Library::default()),
			book: LazyHash::new(book),
			fonts,
			base_path,
			files: RwLock::new(HashMap::new()),
			sources: RwLock::new(HashMap::new()),
			package_manager,
		})
	}

	/// Resolve a file path, handling both local files and package files
	fn resolve_path(&self, id: FileId) -> FileResult<PathBuf> {
		match id.root() {
			VirtualRoot::Package(package) => self.resolve_package_path(package, id.vpath()),
			VirtualRoot::Project => id.vpath().realize(&self.base_path).map_err(|_| FileError::NotFound(PathBuf::from(id.vpath().get_with_slash()))),
		}
	}

	/// Resolve a path within a package
	fn resolve_package_path(&self, package: &TypstPackageSpec, vpath: &VirtualPath) -> FileResult<PathBuf> {
		// Convert Typst PackageSpec to our PackageSpec
		let spec = PackageSpec { namespace: package.namespace.to_string(), name: package.name.to_string(), version: package.version.to_string() };

		// Get the package (downloading if necessary)
		let package_dir = self.package_manager.get_package(&spec).map_err(|e| {
			debug!("Failed to get package {}: {}", spec.name, e);
			FileError::NotFound(PathBuf::from(format!("@{}/{}", spec.namespace, spec.name)))
		})?;

		// Resolve the file within the package
		let relative_path = vpath.get_with_slash();
		let full_path = package_dir.join(relative_path);

		if full_path.exists() {
			Ok(full_path)
		} else {
			debug!("Package file not found: {:?}", full_path);
			Err(FileError::NotFound(full_path))
		}
	}
}

impl World for Md2PdfWorld {
	fn library(&self) -> &LazyHash<Library> {
		&self.library
	}

	fn book(&self) -> &LazyHash<FontBook> {
		&self.book
	}

	fn main(&self) -> FileId {
		self.main_source.id()
	}

	fn source(&self, id: FileId) -> FileResult<Source> {
		// Return main source if it matches
		if id == self.main_source.id() {
			return Ok(self.main_source.clone());
		}

		// Check cache
		if let Ok(sources) = self.sources.read()
			&& let Some(source) = sources.get(&id)
		{
			return Ok(source.clone());
		}

		// Resolve the path (handles both local and package files)
		let path = self.resolve_path(id)?;

		let content = std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;

		let source = Source::new(id, content);

		// Cache the source
		if let Ok(mut sources) = self.sources.write() {
			sources.insert(id, source.clone());
		}

		Ok(source)
	}

	fn file(&self, id: FileId) -> FileResult<Bytes> {
		// Check cache first
		if let Ok(files) = self.files.read()
			&& let Some(bytes) = files.get(&id)
		{
			return Ok(bytes.clone());
		}

		// Resolve the path (handles both local and package files)
		let path = self.resolve_path(id)?;

		let bytes = std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;

		let bytes = Bytes::new(bytes);

		// Cache the file
		if let Ok(mut files) = self.files.write() {
			files.insert(id, bytes.clone());
		}

		Ok(bytes)
	}

	fn font(&self, index: usize) -> Option<Font> {
		self.fonts.get(index).cloned()
	}

	fn today(&self, offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
		use chrono::{Datelike, Utc};

		let seconds = offset.map_or(0.0, |d| d.seconds());
		let datetime = Utc::now() + chrono::Duration::seconds(seconds as i64);

		Datetime::from_ymd(datetime.year(), datetime.month().try_into().ok()?, datetime.day().try_into().ok()?)
	}
}

/// Load fonts from typst-assets and system
fn load_fonts() -> (FontBook, Vec<Font>) {
	let mut book = FontBook::new();
	let mut fonts = Vec::new();

	// Load embedded fonts from typst-assets
	for data in typst_assets::fonts() {
		let buffer = Bytes::new(data);
		for font in Font::iter(buffer) {
			book.push(font.info().clone());
			fonts.push(font);
		}
	}

	// Optionally load system fonts
	load_system_fonts(&mut book, &mut fonts);

	(book, fonts)
}

/// Load system fonts
fn load_system_fonts(book: &mut FontBook, fonts: &mut Vec<Font>) {
	// Try to find common font directories
	let font_dirs: Vec<PathBuf> = get_system_font_dirs();

	for dir in font_dirs {
		if dir.exists() {
			load_fonts_from_dir(&dir, book, fonts);
		}
	}
}

fn get_system_font_dirs() -> Vec<PathBuf> {
	let mut dirs = Vec::new();

	#[cfg(target_os = "windows")]
	{
		if let Some(windir) = std::env::var_os("WINDIR") {
			dirs.push(PathBuf::from(windir).join("Fonts"));
		}
		if let Some(localappdata) = std::env::var_os("LOCALAPPDATA") {
			dirs.push(PathBuf::from(localappdata).join("Microsoft\\Windows\\Fonts"));
		}
	}

	#[cfg(target_os = "macos")]
	{
		dirs.push(PathBuf::from("/System/Library/Fonts"));
		dirs.push(PathBuf::from("/Library/Fonts"));
		if let Some(home) = std::env::var_os("HOME") {
			dirs.push(PathBuf::from(home).join("Library/Fonts"));
		}
	}

	#[cfg(target_os = "linux")]
	{
		dirs.push(PathBuf::from("/usr/share/fonts"));
		dirs.push(PathBuf::from("/usr/local/share/fonts"));
		if let Some(home) = std::env::var_os("HOME") {
			dirs.push(PathBuf::from(home.clone()).join(".fonts"));
			dirs.push(PathBuf::from(home).join(".local/share/fonts"));
		}
	}

	dirs
}

fn load_fonts_from_dir(dir: &Path, book: &mut FontBook, fonts: &mut Vec<Font>) {
	let Ok(entries) = std::fs::read_dir(dir) else {
		return;
	};

	for entry in entries.flatten() {
		let path = entry.path();

		if path.is_dir() {
			// Recurse into subdirectories
			load_fonts_from_dir(&path, book, fonts);
		} else if is_font_file(&path)
			&& let Ok(data) = std::fs::read(&path)
		{
			let buffer = Bytes::new(data);
			for font in Font::iter(buffer) {
				book.push(font.info().clone());
				fonts.push(font);
			}
		}
	}
}

fn is_font_file(path: &Path) -> bool {
	matches!(path.extension().and_then(|s| s.to_str()), Some("ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple_pdf_generation() {
		let typst_content = r#"
#set page(paper: "a4")
#set text(size: 12pt)

= Hello World

This is a test document.
"#;

		let result = generate_pdf(typst_content, None);
		assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

		let pdf_bytes = result.unwrap();
		assert!(!pdf_bytes.is_empty());
		assert!(pdf_bytes.starts_with(b"%PDF"));
	}
}
