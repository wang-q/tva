//! Core TSV parsing and manipulation primitives.
//!
//! This module provides the low-level building blocks for reading, writing, and
//! manipulating TSV data. It includes:
//!
//! - **Reader**: Fast, zero-copy, buffered TSV reading.
//! - **Record**: Efficient row representation.
//! - **Fields**: Parsing of field selection specs (e.g. `1,3-5`).
//! - **Select**: High-performance field selection logic.
//! - **Split**: SIMD-accelerated line splitting.
//! - **Key**: Key extraction for grouping and joining.
//! - **Header**: Header detection and handling.

pub mod fields;
pub mod header;
pub mod key;
pub mod reader;
pub mod record;
pub mod select;
pub mod split;

/// Full conventions document included at compile time.
const CONVENTIONS_FULL: &str = include_str!("../../../docs/conventions.md");

/// Help text for field selection syntax, extracted from docs/conventions.md.
pub static FIELD_SYNTAX_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        extract_markdown_section(CONVENTIONS_FULL, "Field Selection Syntax")
    });

/// Help text for header handling, extracted from docs/conventions.md.
pub static HEADER_HELP: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    extract_markdown_section(CONVENTIONS_FULL, "Header Handling")
});

/// Extracts a section from a markdown document by its header.
///
/// Finds the section starting with `## {section_name}` and returns everything
/// from that header up to (but not including) the next `## ` header.
pub fn extract_markdown_section(content: &str, section_name: &str) -> String {
    let needle = format!("## {}", section_name);

    // Find the section start
    let start = match content.find(&needle) {
        Some(pos) => pos,
        None => return format!("# {}\n\nDocumentation not found.\n", section_name),
    };

    // Find the end of the section (start of next ## section)
    let rest = &content[start + needle.len()..];
    let end_offset = rest.find("\n## ").unwrap_or(rest.len());

    // Extract the section
    content[start..start + needle.len() + end_offset].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_section_found() {
        let content = "## Section A\nContent A\n## Section B\nContent B";
        let result = extract_markdown_section(content, "Section A");
        assert!(result.contains("## Section A"));
        assert!(result.contains("Content A"));
        assert!(!result.contains("Section B"));
    }

    #[test]
    fn test_extract_markdown_section_not_found() {
        let content = "## Section A\nContent A";
        let result = extract_markdown_section(content, "NonExistent");
        assert_eq!(result, "# NonExistent\n\nDocumentation not found.\n");
    }

    #[test]
    fn test_extract_markdown_section_last_section() {
        // Section at the end with no following section
        let content = "## Section A\nContent A\n## Section B\nContent B";
        let result = extract_markdown_section(content, "Section B");
        assert!(result.contains("## Section B"));
        assert!(result.contains("Content B"));
    }
}
