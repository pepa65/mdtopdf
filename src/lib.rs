//! mdtopdf - Convert Markdown to PDF with Typst
//!
//! This library provides functionality to convert Markdown documents
//! to PDF using Typst as the rendering engine.

pub mod cli;
pub mod converter;
pub mod parser;
pub mod pdf;

pub use parser::frontmatter::DocumentMetadata;
pub use parser::markdown::ParsedDocument;
pub use pdf::generate_pdf;
