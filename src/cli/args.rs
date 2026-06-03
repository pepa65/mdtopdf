//! Command-line argument definitions using Clap

use clap::Parser;
use std::path::PathBuf;

/// md2pdf - Convert Markdown to PDF using Typst
#[derive(Parser, Debug)]
#[command(name = "md2pdf")]
#[command(author = "md2pdf contributors")]
#[command(version = "0.1.0")]
#[command(about = "Convert Markdown to PDF using Typst", long_about = None)]
pub struct Args {
    /// Input markdown file
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    /// Output PDF file (defaults to input name with .pdf extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Font family to use
    #[arg(long, default_value = "Inter")]
    pub font: String,

    /// Font size in points
    #[arg(long, default_value = "11")]
    pub font_size: u32,

    /// Paper format (A4, Letter)
    #[arg(long, default_value = "A4")]
    pub format: String,

    /// Generation timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u32,

    /// Generate table of contents
    #[arg(long)]
    pub toc: bool,

    /// Generate cover page
    #[arg(long)]
    pub cover: bool,

    /// Header left text
    #[arg(long)]
    pub header_left: Option<String>,

    /// Header center text
    #[arg(long)]
    pub header_center: Option<String>,

    /// Header right text
    #[arg(long)]
    pub header_right: Option<String>,

    /// Footer left text
    #[arg(long)]
    pub footer_left: Option<String>,

    /// Footer center text
    #[arg(long)]
    pub footer_center: Option<String>,

    /// Footer right text
    #[arg(long)]
    pub footer_right: Option<String>,

    /// Disable page numbers
    #[arg(long)]
    pub no_page_numbers: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// Determine the output path based on input and optional override
    pub fn get_output_path(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| {
            let mut path = self.input.clone();
            path.set_extension("pdf");
            path
        })
    }
}
