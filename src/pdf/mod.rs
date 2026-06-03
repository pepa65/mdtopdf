//! PDF generation module using Typst

pub mod generator;
pub mod packages;

pub use generator::generate_pdf;
pub use packages::{PackageManager, PackageSpec};
