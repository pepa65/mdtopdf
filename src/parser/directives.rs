//! Custom directive parsing for layout and page control

use std::fmt;

/// Custom directives that can appear in markdown content
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// Force a page break
    PageBreak,
    /// Page break before the next element
    PageBreakBefore,
    /// Page break after the current element
    PageBreakAfter,

    /// Start a 2-column layout
    Columns2,
    /// Start a 3-column layout
    Columns3,
    /// Start a 4-column layout
    Columns4,
    /// End column layout
    ColumnsEnd,

    /// Two-column grid layout
    TwoColumn,
    /// Three-column grid layout
    ThreeColumn,
    /// Sidebar layout (narrow left, wide right)
    SidebarLayout,
    /// End grid layout
    GridEnd,

    /// Keep following content together (no page break)
    KeepTogether,
    /// Alias for KeepTogether
    NoBreak,
    /// End keep together block
    KeepTogetherEnd,

    /// Set column widths for the next table
    /// The value is the Typst column specification, e.g., "1fr, 2fr, 1fr" or "auto, 100pt, 1fr"
    TableColumns(String),
}

impl Directive {
    /// Try to parse a directive from text content
    pub fn parse(text: &str) -> Option<Directive> {
        let text = text.trim();

        // Check for bracketed directives [directive-name]
        if !text.starts_with('[') || !text.ends_with(']') {
            return None;
        }

        let inner = &text[1..text.len() - 1];

        // Check for directives with values (e.g., [table-columns: 1fr, 2fr])
        if let Some(colon_pos) = inner.find(':') {
            let key = inner[..colon_pos].trim().to_lowercase();
            let value = inner[colon_pos + 1..].trim().to_string();

            return match key.as_str() {
                "table-columns" | "table-cols" | "cols" => {
                    if value.is_empty() {
                        None
                    } else {
                        Some(Directive::TableColumns(value))
                    }
                }
                _ => None,
            };
        }

        let inner_lower = inner.to_lowercase();

        match inner_lower.as_str() {
            "page-break" | "pagebreak" => Some(Directive::PageBreak),
            "page-break-before" | "pagebreak-before" => Some(Directive::PageBreakBefore),
            "page-break-after" | "pagebreak-after" => Some(Directive::PageBreakAfter),

            "columns-2" | "columns2" | "2-columns" => Some(Directive::Columns2),
            "columns-3" | "columns3" | "3-columns" => Some(Directive::Columns3),
            "columns-4" | "columns4" | "4-columns" => Some(Directive::Columns4),
            "/columns" | "/columns-2" | "/columns-3" | "/columns-4"
            | "columns-end" | "end-columns" => Some(Directive::ColumnsEnd),

            "two-column" | "twocolumn" => Some(Directive::TwoColumn),
            "three-column" | "threecolumn" => Some(Directive::ThreeColumn),
            "sidebar-layout" | "sidebar" => Some(Directive::SidebarLayout),
            "/grid" | "grid-end" | "end-grid" => Some(Directive::GridEnd),

            "keep-together" | "keeptogether" => Some(Directive::KeepTogether),
            "no-break" | "nobreak" => Some(Directive::NoBreak),
            "/keep-together" | "/keeptogether" | "/no-break" | "/nobreak" => {
                Some(Directive::KeepTogetherEnd)
            }

            _ => None,
        }
    }

    /// Convert directive to Typst code
    pub fn to_typst(&self) -> String {
        match self {
            Directive::PageBreak => "#pagebreak()".to_string(),
            Directive::PageBreakBefore => "#pagebreak()".to_string(),
            Directive::PageBreakAfter => "".to_string(), // Handled differently
            Directive::Columns2 => "#columns(2)[".to_string(),
            Directive::Columns3 => "#columns(3)[".to_string(),
            Directive::Columns4 => "#columns(4)[".to_string(),
            Directive::ColumnsEnd => "]".to_string(),
            Directive::TwoColumn => "#grid(columns: (1fr, 1fr), gutter: 1em)[".to_string(),
            Directive::ThreeColumn => "#grid(columns: (1fr, 1fr, 1fr), gutter: 1em)[".to_string(),
            Directive::SidebarLayout => "#grid(columns: (1fr, 2fr), gutter: 1em)[".to_string(),
            Directive::GridEnd => "]".to_string(),
            Directive::KeepTogether | Directive::NoBreak => "#block(breakable: false)[".to_string(),
            Directive::KeepTogetherEnd => "]".to_string(),
            // TableColumns doesn't output directly - it's handled by the TypstBuilder
            Directive::TableColumns(_) => "".to_string(),
        }
    }
}

impl Directive {
    /// Get the table columns specification if this is a TableColumns directive
    pub fn table_columns(&self) -> Option<&str> {
        match self {
            Directive::TableColumns(cols) => Some(cols),
            _ => None,
        }
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_typst())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_break() {
        assert_eq!(Directive::parse("[page-break]"), Some(Directive::PageBreak));
        assert_eq!(Directive::parse("[pagebreak]"), Some(Directive::PageBreak));
        assert_eq!(
            Directive::parse("[PAGE-BREAK]"),
            Some(Directive::PageBreak)
        );
    }

    #[test]
    fn test_parse_columns() {
        assert_eq!(Directive::parse("[columns-2]"), Some(Directive::Columns2));
        assert_eq!(Directive::parse("[columns-3]"), Some(Directive::Columns3));
        assert_eq!(Directive::parse("[/columns]"), Some(Directive::ColumnsEnd));
    }

    #[test]
    fn test_parse_grid_layouts() {
        assert_eq!(
            Directive::parse("[two-column]"),
            Some(Directive::TwoColumn)
        );
        assert_eq!(
            Directive::parse("[three-column]"),
            Some(Directive::ThreeColumn)
        );
        assert_eq!(
            Directive::parse("[sidebar-layout]"),
            Some(Directive::SidebarLayout)
        );
    }

    #[test]
    fn test_parse_keep_together() {
        assert_eq!(
            Directive::parse("[keep-together]"),
            Some(Directive::KeepTogether)
        );
        assert_eq!(Directive::parse("[no-break]"), Some(Directive::NoBreak));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(Directive::parse("not a directive"), None);
        assert_eq!(Directive::parse("[unknown]"), None);
        assert_eq!(Directive::parse("page-break"), None); // Missing brackets
    }

    #[test]
    fn test_to_typst() {
        assert_eq!(Directive::PageBreak.to_typst(), "#pagebreak()");
        assert_eq!(Directive::Columns2.to_typst(), "#columns(2)[");
        assert_eq!(Directive::ColumnsEnd.to_typst(), "]");
    }

    #[test]
    fn test_parse_table_columns() {
        assert_eq!(
            Directive::parse("[table-columns: 1fr, 2fr, 1fr]"),
            Some(Directive::TableColumns("1fr, 2fr, 1fr".to_string()))
        );
        assert_eq!(
            Directive::parse("[table-cols: auto, 100pt, 1fr]"),
            Some(Directive::TableColumns("auto, 100pt, 1fr".to_string()))
        );
        assert_eq!(
            Directive::parse("[cols: 25%, 50%, 25%]"),
            Some(Directive::TableColumns("25%, 50%, 25%".to_string()))
        );
        // Empty value should return None
        assert_eq!(Directive::parse("[table-columns:]"), None);
        assert_eq!(Directive::parse("[table-columns:   ]"), None);
    }

    #[test]
    fn test_table_columns_helper() {
        let directive = Directive::TableColumns("1fr, 2fr".to_string());
        assert_eq!(directive.table_columns(), Some("1fr, 2fr"));

        let other = Directive::PageBreak;
        assert_eq!(other.table_columns(), None);
    }
}
