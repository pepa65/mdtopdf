//! Command-line argument definitions using Clap

use clap::Parser;
use std::path::PathBuf;

use crate::parser::frontmatter::{Margins, parse_margins};

/// mdtopdf - Convert Markdown to PDF with Typst
#[derive(Parser, Debug)]
#[clap(version, about)]
#[clap(help_template(
	"\
{name} {version} - {about}
{usage-heading} {usage}
{all-args}
"
))]
pub struct Args {
	/// Input markdown file
	#[arg(value_name = "MD_FILE", required_unless_present = "readme")]
	pub input: Option<PathBuf>,

	/// Output PDF file [default: MD_FILE with .pdf extension]
	#[arg(short, long, value_name = "PDF_FILE")]
	pub output: Option<PathBuf>,

	/// Font family to use
	#[arg(short, long, default_value = "Inter")]
	pub font: String,

	/// Font size in points (can be fractional)
	#[arg(short = 's', long, default_value = "11")]
	pub font_size: f32,

	/// Paper format (A4/Letter)
	#[arg(short, long, default_value = "A4")]
	pub paper: String,

	/// Page margins: `top [right [bottom [left]]] [mm|cm|in|pt]` [default: 16 mm]
	#[arg(short = 'm', long = "margins", num_args = 1..=5)]
	pub margins: Option<Vec<String>>,

	/// Generate Table of Contents
	#[arg(long)]
	pub toc: bool,

	/// Header for Table of Contents
	#[arg(long, default_value = "Table of Contents")]
	pub toc_header: Option<String>,

	/// Generate Cover page
	#[arg(long)]
	pub cover: bool,

	/// Header Left text
	#[arg(long)]
	pub header_left: Option<String>,

	/// Header Center text
	#[arg(long)]
	pub header_center: Option<String>,

	/// Header Right text
	#[arg(long)]
	pub header_right: Option<String>,

	/// Footer Left text
	#[arg(long)]
	pub footer_left: Option<String>,

	/// Footer Center text
	#[arg(long)]
	pub footer_center: Option<String>,

	/// Footer Right text
	#[arg(long)]
	pub footer_right: Option<String>,

	/// Disable page numbers
	#[arg(long)]
	pub no_page_numbers: bool,

	/// Output readme file from repo
	#[arg(short, long)]
	pub readme: bool,

	/// Enable verbose output
	#[arg(short, long)]
	pub verbose: bool,
}

impl Args {
	pub fn margins(&self) -> Result<Option<Margins>, String> {
		self.margins
			.as_ref()
			.map(|values| {
				let parts: Vec<&str> = values.iter().map(String::as_str).collect();
				parse_margins(&parts)
			})
			.transpose()
	}
}
