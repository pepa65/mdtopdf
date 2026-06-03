use anyhow::Result;
use clap::Parser;
use tracing::{info, debug, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod converter;
mod parser;
mod pdf;

use cli::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    // Set up logging
    let log_level = if args.verbose { Level::DEBUG } else { Level::INFO };
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("md2pdf - Markdown to PDF converter");

    // Read the input file
    let content = cli::read_input_file(&args.input)?;
    info!("Read {} bytes from {:?}", content.len(), args.input);

    // Parse frontmatter and markdown
    let (metadata, markdown_content) = parser::frontmatter::extract_frontmatter(&content)?;
    info!("Parsed frontmatter: {:?}", metadata.title);

    // Parse markdown and extract directives
    let document = parser::markdown::parse_markdown(&markdown_content)?;
    info!("Parsed {} markdown events", document.events.len());

    // Set up mermaid cache directory in the source file's parent directory
    let base_path = args.input.parent().map(|p| p.to_path_buf());
    let mermaid_cache_dir = base_path.as_ref().map(|p| p.join(".mermaid"));

    // Create mermaid cache directory if needed
    if let Some(ref cache_dir) = mermaid_cache_dir {
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            debug!("Could not create mermaid cache dir: {}", e);
        }
    }

    // Convert to Typst (with optional mermaid rendering via kroki.io)
    let typst_content = converter::typst_builder::convert_to_typst_with_cache(
        &document,
        &metadata,
        mermaid_cache_dir.as_deref(),
    )?;
    debug!("Generated Typst content ({} bytes)", typst_content.len());

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("pdf");
        path
    });

    // Debug: write Typst content if verbose
    if args.verbose {
        let mut typst_path = output_path.clone();
        typst_path.set_extension("typ");
        std::fs::write(&typst_path, &typst_content)?;
        debug!("Wrote Typst debug output to {:?}", typst_path);
    }

    // Generate PDF
    info!("Generating PDF...");
    let pdf_bytes = pdf::generate_pdf(&typst_content, base_path.as_deref())?;
    info!("Generated PDF ({} bytes)", pdf_bytes.len());

    // Write PDF to file
    std::fs::write(&output_path, &pdf_bytes)?;
    info!("Wrote PDF to {:?}", output_path);

    println!("Successfully converted {:?} to {:?}", args.input, output_path);

    Ok(())
}
