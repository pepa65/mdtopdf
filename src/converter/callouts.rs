//! Callout box detection and conversion
//!
//! Converts emoji-prefixed blockquotes or GitHub-style alerts to styled callout boxes.
//!
//! Supported formats:
//! - Emoji prefix: `> 💡 This is a tip`
//! - GitHub alerts: `> [!TIP]` or `> [!WARNING]`
//! - Text prefix: `> **Tip:** This is a tip`

use std::collections::HashMap;

/// Types of callout boxes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalloutType {
    Tip,
    Warning,
    Success,
    Error,
    Note,
    Details,
    Info,
    Important,
    Caution,
}

impl CalloutType {
    /// Get the display name for the callout type
    pub fn display_name(&self) -> &'static str {
        match self {
            CalloutType::Tip => "Tip",
            CalloutType::Warning => "Warning",
            CalloutType::Success => "Success",
            CalloutType::Error => "Error",
            CalloutType::Note => "Note",
            CalloutType::Details => "Details",
            CalloutType::Info => "Info",
            CalloutType::Important => "Important",
            CalloutType::Caution => "Caution",
        }
    }

    /// Get the background color for the callout type
    pub fn background_color(&self) -> &'static str {
        match self {
            CalloutType::Tip => "#e7f3ff",
            CalloutType::Warning => "#fff3cd",
            CalloutType::Success => "#d4edda",
            CalloutType::Error => "#f8d7da",
            CalloutType::Note => "#e2e3e5",
            CalloutType::Details => "#e2d5f0",
            CalloutType::Info => "#cfe2ff",
            CalloutType::Important => "#f8d7da",
            CalloutType::Caution => "#fff3cd",
        }
    }

    /// Get the border/accent color for the callout type
    pub fn border_color(&self) -> &'static str {
        match self {
            CalloutType::Tip => "#0066cc",
            CalloutType::Warning => "#cc8800",
            CalloutType::Success => "#28a745",
            CalloutType::Error => "#dc3545",
            CalloutType::Note => "#6c757d",
            CalloutType::Details => "#6f42c1",
            CalloutType::Info => "#0d6efd",
            CalloutType::Important => "#dc3545",
            CalloutType::Caution => "#cc8800",
        }
    }

    /// Get the icon/emoji for the callout type
    pub fn icon(&self) -> &'static str {
        match self {
            CalloutType::Tip => "💡",
            CalloutType::Warning => "⚠️",
            CalloutType::Success => "✅",
            CalloutType::Error => "❌",
            CalloutType::Note => "📝",
            CalloutType::Details => "📋",
            CalloutType::Info => "ℹ️",
            CalloutType::Important => "❗",
            CalloutType::Caution => "⚠️",
        }
    }
}

/// Parsed callout information
#[derive(Debug, Clone)]
pub struct CalloutInfo {
    /// The type of callout
    pub callout_type: CalloutType,
    /// Optional custom title (overrides default)
    pub title: Option<String>,
    /// The content of the callout (without the type marker)
    pub content: String,
}

/// Mapping of emoji to callout types
fn emoji_to_callout() -> HashMap<&'static str, CalloutType> {
    let mut map = HashMap::new();
    map.insert("💡", CalloutType::Tip);
    map.insert("⚡", CalloutType::Tip);
    map.insert("🔥", CalloutType::Tip);
    map.insert("⚠️", CalloutType::Warning);
    map.insert("⚠", CalloutType::Warning);
    map.insert("⛔", CalloutType::Warning);
    map.insert("✅", CalloutType::Success);
    map.insert("✓", CalloutType::Success);
    map.insert("👍", CalloutType::Success);
    map.insert("❌", CalloutType::Error);
    map.insert("🚫", CalloutType::Error);
    map.insert("❗", CalloutType::Error);
    map.insert("📝", CalloutType::Note);
    map.insert("📌", CalloutType::Note);
    map.insert("📋", CalloutType::Details);
    map.insert("ℹ️", CalloutType::Info);
    map.insert("ℹ", CalloutType::Info);
    map.insert("🔔", CalloutType::Important);
    map
}

/// Try to detect if text is a callout and extract its info
///
/// Returns `None` if the text is not a callout.
pub fn detect_callout(text: &str) -> Option<CalloutInfo> {
    let text = text.trim();

    // Try GitHub-style alerts: [!TYPE] or [!TYPE] Title
    if let Some(info) = detect_github_alert(text) {
        return Some(info);
    }

    // Try emoji prefix
    if let Some(info) = detect_emoji_callout(text) {
        return Some(info);
    }

    // Try text prefix (e.g., **Tip:** or **Warning:**)
    if let Some(info) = detect_text_prefix_callout(text) {
        return Some(info);
    }

    None
}

/// Detect GitHub-style alert: `[!TIP]` or `[!WARNING] Custom Title`
fn detect_github_alert(text: &str) -> Option<CalloutInfo> {
    let text = text.trim();

    if !text.starts_with("[!") {
        return None;
    }

    let close_bracket = text.find(']')?;
    let alert_type = &text[2..close_bracket];

    let callout_type = match alert_type.to_uppercase().as_str() {
        "TIP" => CalloutType::Tip,
        "WARNING" | "WARN" => CalloutType::Warning,
        "SUCCESS" | "CHECK" => CalloutType::Success,
        "ERROR" | "DANGER" | "FAILURE" => CalloutType::Error,
        "NOTE" => CalloutType::Note,
        "DETAILS" => CalloutType::Details,
        "INFO" => CalloutType::Info,
        "IMPORTANT" => CalloutType::Important,
        "CAUTION" => CalloutType::Caution,
        _ => return None,
    };

    let rest = text[close_bracket + 1..].trim();

    // Check if there's a custom title on the same line
    let (title, content) = if let Some(newline_pos) = rest.find('\n') {
        let first_line = rest[..newline_pos].trim();
        let remaining = rest[newline_pos + 1..].trim();

        if first_line.is_empty() {
            (None, remaining.to_string())
        } else {
            (Some(first_line.to_string()), remaining.to_string())
        }
    } else if rest.is_empty() {
        (None, String::new())
    } else {
        // Single line - could be title or content
        (None, rest.to_string())
    };

    Some(CalloutInfo {
        callout_type,
        title,
        content,
    })
}

/// Detect emoji-prefixed callout
fn detect_emoji_callout(text: &str) -> Option<CalloutInfo> {
    let emoji_map = emoji_to_callout();

    for (emoji, callout_type) in &emoji_map {
        if text.starts_with(emoji) {
            let content = text[emoji.len()..].trim().to_string();
            return Some(CalloutInfo {
                callout_type: *callout_type,
                title: None,
                content,
            });
        }
    }

    None
}

/// Detect text-prefixed callout like `**Tip:**` or `**Warning:**`
fn detect_text_prefix_callout(text: &str) -> Option<CalloutInfo> {
    let prefixes = [
        ("**Tip:**", CalloutType::Tip),
        ("**Warning:**", CalloutType::Warning),
        ("**Success:**", CalloutType::Success),
        ("**Error:**", CalloutType::Error),
        ("**Note:**", CalloutType::Note),
        ("**Details:**", CalloutType::Details),
        ("**Info:**", CalloutType::Info),
        ("**Important:**", CalloutType::Important),
        ("**Caution:**", CalloutType::Caution),
        // Also support without bold
        ("Tip:", CalloutType::Tip),
        ("Warning:", CalloutType::Warning),
        ("Success:", CalloutType::Success),
        ("Error:", CalloutType::Error),
        ("Note:", CalloutType::Note),
        ("Details:", CalloutType::Details),
        ("Info:", CalloutType::Info),
        ("Important:", CalloutType::Important),
        ("Caution:", CalloutType::Caution),
    ];

    for (prefix, callout_type) in prefixes {
        if text.starts_with(prefix) {
            let content = text[prefix.len()..].trim().to_string();
            return Some(CalloutInfo {
                callout_type,
                title: None,
                content,
            });
        }
    }

    None
}

/// Generate Typst code for a callout box
pub fn generate_callout_typst(info: &CalloutInfo) -> String {
    let title = info
        .title
        .as_ref()
        .map(|t| t.as_str())
        .unwrap_or_else(|| info.callout_type.display_name());

    let icon = info.callout_type.icon();
    let bg_color = info.callout_type.background_color();
    let border_color = info.callout_type.border_color();

    format!(
        r#"#block(
  fill: rgb("{}"),
  stroke: (left: 4pt + rgb("{}")),
  inset: 1em,
  radius: (right: 4pt),
  width: 100%,
)[
  #text(weight: "bold")[{} {}]
  #parbreak()
  {}
]

"#,
        bg_color, border_color, icon, title, info.content
    )
}

/// Preamble function definition for callouts (to include in document preamble)
pub fn callout_preamble() -> &'static str {
    r##"// Callout box helper
#let callout(type: "note", title: none, icon: none, body) = {
  let styles = (
    tip: (bg: rgb("#e7f3ff"), border: rgb("#0066cc"), icon: [💡]),
    warning: (bg: rgb("#fff3cd"), border: rgb("#cc8800"), icon: [⚠️]),
    success: (bg: rgb("#d4edda"), border: rgb("#28a745"), icon: [✅]),
    error: (bg: rgb("#f8d7da"), border: rgb("#dc3545"), icon: [❌]),
    note: (bg: rgb("#e2e3e5"), border: rgb("#6c757d"), icon: [📝]),
    details: (bg: rgb("#e2d5f0"), border: rgb("#6f42c1"), icon: [📋]),
    info: (bg: rgb("#cfe2ff"), border: rgb("#0d6efd"), icon: [ℹ️]),
    important: (bg: rgb("#f8d7da"), border: rgb("#dc3545"), icon: [❗]),
    caution: (bg: rgb("#fff3cd"), border: rgb("#cc8800"), icon: [⚠️]),
  )
  let style = styles.at(type, default: styles.note)
  let display-icon = if icon != none { icon } else { style.icon }
  let display-title = if title != none { title } else { upper(type.at(0)) + type.slice(1) }

  block(
    fill: style.bg,
    stroke: (left: 4pt + style.border),
    inset: 1em,
    radius: (right: 4pt),
    width: 100%,
  )[
    #text(weight: "bold")[#display-icon #display-title]
    #parbreak()
    #body
  ]
}

"##
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_github_alert_tip() {
        let info = detect_callout("[!TIP]\nThis is a tip").unwrap();
        assert_eq!(info.callout_type, CalloutType::Tip);
        assert_eq!(info.content, "This is a tip");
    }

    #[test]
    fn test_detect_github_alert_with_title() {
        let info = detect_callout("[!WARNING] Custom Title\nThis is the content").unwrap();
        assert_eq!(info.callout_type, CalloutType::Warning);
        assert_eq!(info.title, Some("Custom Title".to_string()));
        assert_eq!(info.content, "This is the content");
    }

    #[test]
    fn test_detect_emoji_callout() {
        let info = detect_callout("💡 This is a tip").unwrap();
        assert_eq!(info.callout_type, CalloutType::Tip);
        assert_eq!(info.content, "This is a tip");
    }

    #[test]
    fn test_detect_text_prefix_callout() {
        let info = detect_callout("**Warning:** Be careful!").unwrap();
        assert_eq!(info.callout_type, CalloutType::Warning);
        assert_eq!(info.content, "Be careful!");
    }

    #[test]
    fn test_no_callout() {
        assert!(detect_callout("Just a regular quote").is_none());
    }
}
