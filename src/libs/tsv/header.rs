//! Header detection and handling for TSV data.
//!
//! This module provides unified header processing logic for all tva commands.
//!
//! # Architecture
//!
//! The module provides three main types for different use cases:
//!
//! - [`HeaderConfig`]: Configuration for header detection modes (CLI layer)
//! - [`HeaderHandler`]: Streaming processor for capturing headers during line-by-line processing
//! - [`Header`]: Unified header structure with field resolution capabilities
//!
//! # Usage Examples
//!
//! ## Streaming processing with HeaderHandler
//!
//! ```rust
//! use tva::libs::tsv::header::{HeaderConfig, HeaderHandler};
//!
//! let config = HeaderConfig::new().enabled().first_line();
//! let mut handler = HeaderHandler::new(config);
//!
//! // Process first line - captures it as header
//! let is_header = handler.process_first_line(b"col1\tcol2").unwrap();
//! assert!(is_header);
//!
//! // Subsequent lines are data
//! let is_header = handler.process_first_line(b"val1\tval2").unwrap();
//! assert!(!is_header);
//!
//! // Get captured header
//! assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
//! ```
//!
//! ## Field resolution with Header
//!
//! ```rust
//! use tva::libs::tsv::header::Header;
//!
//! let header = Header::from_column_names(b"name\tage\tcity".to_vec(), '\t');
//!
//! // Look up field index by name (0-based)
//! assert_eq!(header.get_index("age"), Some(1));
//! assert_eq!(header.get_index("name"), Some(0));
//! ```
//!
//! ## Writing header to output
//!
//! ```rust
//! use tva::libs::tsv::header::{Header, write_header};
//!
//! let header = Header::from_column_names(b"col1\tcol2".to_vec(), '\t');
//! let mut output = Vec::new();
//! write_header(&mut output, &header, None).unwrap();
//! assert_eq!(output, b"col1\tcol2\n");
//! ```

use std::collections::HashMap;
use std::io::{self, Write};

/// Header detection mode. The four modes are mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMode {
    /// Use the first line as header (default), even if empty.
    FirstLine,
    /// Use exactly N lines as header (including empty lines).
    LinesN(usize),
    /// Use consecutive hash lines (starting with '#') as header.
    HashLines,
    /// Use consecutive hash lines plus the next line as header (for extracting column names).
    HashLines1,
}

/// Configuration for header detection.
#[derive(Debug, Clone)]
pub struct HeaderConfig {
    /// Whether header processing is enabled.
    pub enabled: bool,
    /// The header detection mode.
    pub mode: HeaderMode,
}

impl HeaderConfig {
    /// Creates a new header config with default settings (disabled, FirstLine mode).
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: HeaderMode::FirstLine,
        }
    }

    /// Enables header processing.
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Sets the mode to FirstLine (use first line as header, even if empty).
    pub fn first_line(mut self) -> Self {
        self.mode = HeaderMode::FirstLine;
        self
    }

    /// Sets the mode to LinesN (use exactly N lines as header, including empty lines).
    pub fn lines_n(mut self, n: usize) -> Self {
        self.mode = HeaderMode::LinesN(n);
        self
    }

    /// Sets the mode to HashLines (use consecutive '#' lines as header).
    pub fn hash_lines(mut self) -> Self {
        self.mode = HeaderMode::HashLines;
        self
    }

    /// Sets the mode to HashLines1 (use consecutive '#' lines + next line as header).
    pub fn hash_lines1(mut self) -> Self {
        self.mode = HeaderMode::HashLines1;
        self
    }
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a detected header (returned by TsvReader).
///
/// This is an internal type used by `TsvReader` to return header detection results.
/// Commands should use [`Header`] instead for unified header handling.
///
/// The relationship between `lines` and `column_names_line`:
/// - For `FirstLine` mode: `lines` contains 1 line, `column_names_line` is that line
/// - For `LinesN(n)` mode: `lines` contains n lines, `column_names_line` is the last line
/// - For `HashLines` mode: `lines` contains hash lines, `column_names_line` is `None`
/// - For `HashLines1` mode: `lines` contains hash lines + column names, `column_names_line` is the last line
pub struct HeaderInfo {
    /// All header lines (e.g., comment lines, or first N lines).
    pub(crate) lines: Vec<Vec<u8>>,
    /// The specific line containing column names (if applicable).
    pub(crate) column_names_line: Option<Vec<u8>>,
}

/// Header handler for streaming processing.
///
/// This struct is designed for line-by-line TSV processing where you need to
/// capture the header from the first input file and potentially write it to output.
///
/// # Key Features
///
/// - Captures header according to the configured [`HeaderMode`]
/// - Handles multi-file input (only captures header from first file)
/// - Treats empty lines as data (not header) for robust streaming behavior
///
/// # Example
///
/// ```rust
/// use tva::libs::tsv::header::{HeaderConfig, HeaderHandler};
///
/// let config = HeaderConfig::new().enabled().hash_lines();
/// let mut handler = HeaderHandler::new(config);
///
/// // Hash lines are captured as header
/// assert!(handler.process_first_line(b"# Comment").unwrap());
/// assert!(handler.process_first_line(b"# Another comment").unwrap());
///
/// // Non-hash line is data
/// assert!(!handler.process_first_line(b"col1\tcol2").unwrap());
///
/// // Header contains both hash lines joined by '\n'
/// assert_eq!(handler.header(), Some(b"# Comment\n# Another comment".as_slice()));
/// ```
pub struct HeaderHandler {
    /// Header detection configuration.
    config: HeaderConfig,
    /// Captured header content (all header lines joined by '\n').
    captured_header: Option<Vec<u8>>,
    /// Whether we're still processing the first file.
    /// Header is only captured from the first file.
    is_first_file: bool,
    /// For LinesN mode: tracks remaining lines to collect as header.
    lines_n_remaining: usize,
}

impl HeaderHandler {
    /// Creates a new header handler with the given config.
    pub fn new(config: HeaderConfig) -> Self {
        let lines_n_remaining = match config.mode {
            HeaderMode::LinesN(n) => n,
            _ => 0,
        };
        Self {
            config,
            captured_header: None,
            is_first_file: true,
            lines_n_remaining,
        }
    }

    /// Returns true if header processing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Returns true if we should output header to files.
    pub fn should_output_header(&self) -> bool {
        self.config.enabled && self.captured_header.is_some()
    }

    /// Gets the captured header line (if any).
    pub fn header(&self) -> Option<&[u8]> {
        self.captured_header.as_deref()
    }

    /// Processes the first line of a file, capturing header if needed.
    ///
    /// Returns `Ok(true)` if the line was consumed as header,
    /// `Ok(false)` if the line should be processed as data.
    pub fn process_first_line(&mut self, line: &[u8]) -> anyhow::Result<bool> {
        // Empty lines are treated as data (not header)
        // Note: This differs from detect_header() which treats empty lines as valid header
        // This is intentional for streaming mode where we don't know if there are more lines
        if line.is_empty() {
            return Ok(false);
        }

        if !self.config.enabled {
            return Ok(false);
        }

        match self.config.mode {
            HeaderMode::FirstLine => self.process_first_line_mode(line),
            HeaderMode::LinesN(n) => self.process_lines_n_mode(line, n),
            HeaderMode::HashLines => self.process_hash_lines_mode(line, false),
            HeaderMode::HashLines1 => self.process_hash_lines_mode(line, true),
        }
    }

    fn process_first_line_mode(&mut self, line: &[u8]) -> anyhow::Result<bool> {
        // First line is the header (empty lines are already filtered by process_first_line)
        if self.is_first_file {
            self.captured_header = Some(line.to_vec());
            self.is_first_file = false;
            return Ok(true);
        }
        Ok(false)
    }

    fn process_lines_n_mode(&mut self, line: &[u8], _n: usize) -> anyhow::Result<bool> {
        // For streaming, LinesN mode captures the first N lines as header from the first file
        if !self.is_first_file {
            // Not the first file, don't collect header
            return Ok(false);
        }
        if self.lines_n_remaining > 0 {
            if let Some(ref mut h) = self.captured_header {
                h.push(b'\n');
                h.extend_from_slice(line);
            } else {
                self.captured_header = Some(line.to_vec());
            }
            self.lines_n_remaining -= 1;
            return Ok(true);
        }
        // Header collection complete
        self.is_first_file = false;
        Ok(false)
    }

    fn process_hash_lines_mode(
        &mut self,
        line: &[u8],
        include_next_line: bool,
    ) -> anyhow::Result<bool> {
        let is_hash = line.starts_with(b"#");

        if is_hash {
            // Hash lines are part of header
            if self.is_first_file {
                // Capture hash line
                if self.captured_header.is_none() {
                    self.captured_header = Some(Vec::new());
                }
                if let Some(ref mut h) = self.captured_header {
                    if !h.is_empty() {
                        h.push(b'\n');
                    }
                    h.extend_from_slice(line);
                }
            }
            Ok(true)
        } else if include_next_line && self.is_first_file {
            // First non-hash line is also part of header (column names)
            if let Some(ref mut h) = self.captured_header {
                if !h.is_empty() {
                    h.push(b'\n');
                }
                h.extend_from_slice(line);
            } else {
                self.captured_header = Some(line.to_vec());
            }
            self.is_first_file = false;
            Ok(true)
        } else {
            // Not a hash line and we don't need to collect it
            Ok(false)
        }
    }

    /// Marks the end of the first file.
    pub fn end_of_file(&mut self) {
        self.is_first_file = false;
        // Reset lines_n_remaining for subsequent files
        if let HeaderMode::LinesN(n) = self.config.mode {
            self.lines_n_remaining = n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_disabled() {
        let config = HeaderConfig::new(); // disabled
        let mut handler = HeaderHandler::new(config);

        assert!(!handler.is_enabled());
        assert!(!handler.should_output_header());
        assert!(!handler.process_first_line(b"col1\tcol2").unwrap());
        assert!(handler.header().is_none());
    }

    #[test]
    fn test_handler_first_line_mode() {
        let config = HeaderConfig::new().enabled().first_line();
        let mut handler = HeaderHandler::new(config);

        // First line is header
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));

        // Second line is data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    #[test]
    fn test_handler_lines_n_mode() {
        let config = HeaderConfig::new().enabled().lines_n(2);
        let mut handler = HeaderHandler::new(config);

        // First two lines are header
        assert!(handler.process_first_line(b"# comment").unwrap());
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"# comment\ncol1\tcol2".as_slice()));

        // Third line is data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    #[test]
    fn test_handler_hash_lines_mode() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // Hash lines are header
        assert!(handler.process_first_line(b"# comment 1").unwrap());
        assert!(handler.process_first_line(b"# comment 2").unwrap());
        assert_eq!(
            handler.header(),
            Some(b"# comment 1\n# comment 2".as_slice())
        );

        // Non-hash line is data
        assert!(!handler.process_first_line(b"col1\tcol2").unwrap());
    }

    #[test]
    fn test_handler_hash_lines1_three_lines() {
        // HashLines1 mode: '#' lines + next line are header (for column names)
        let config = HeaderConfig::new().enabled().hash_lines1();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"# comment 1").unwrap());
        assert!(handler.process_first_line(b"# comment 2").unwrap());
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(
            handler.header(),
            Some(b"# comment 1\n# comment 2\ncol1\tcol2".as_slice())
        );

        // Subsequent line is data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    #[test]
    fn test_handler_auto_detect_first_line() {
        // Default mode (FirstLine) - first line is header
        let config = HeaderConfig::new().enabled();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));

        // Second line is data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    #[test]
    fn test_handler_first_line_empty_skipped() {
        // HeaderHandler treats empty lines as data (not header)
        // This differs from detect_header behavior
        let config = HeaderConfig::new().enabled();
        let mut handler = HeaderHandler::new(config);

        // Empty line is treated as data
        assert!(!handler.process_first_line(b"").unwrap());
        // First non-empty line is header
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_process_first_line() {
        let config = HeaderConfig::new().enabled();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));

        // Second file's first line should not be treated as header
        handler.end_of_file();
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    // Additional tests for coverage

    #[test]
    fn test_header_config_default() {
        let config: HeaderConfig = Default::default();
        assert!(!config.enabled);
        assert!(matches!(config.mode, HeaderMode::FirstLine));
    }

    #[test]
    fn test_handler_header_as_bytes_single() {
        // HeaderHandler stores header as single line
        let config = HeaderConfig::new().enabled().first_line();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_header_as_bytes_multiple() {
        // HeaderHandler joins multiple header lines with '\n'
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"# comment 1").unwrap());
        assert!(handler.process_first_line(b"# comment 2").unwrap());
        // Non-hash line is data
        assert!(!handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(
            handler.header(),
            Some(b"# comment 1\n# comment 2".as_slice())
        );
    }

    #[test]
    fn test_handler_header_empty() {
        // No header captured when disabled
        let config = HeaderConfig::new(); // disabled
        let handler = HeaderHandler::new(config);

        assert!(handler.header().is_none());
    }

    #[test]
    fn test_handler_first_line_empty_data() {
        // Empty data - no lines processed
        let config = HeaderConfig::new().enabled().first_line();
        let handler = HeaderHandler::new(config);

        assert!(handler.header().is_none());
        assert!(!handler.should_output_header());
    }

    #[test]
    fn test_handler_first_line_only_empty_lines() {
        // HeaderHandler treats empty lines as data, not header
        let config = HeaderConfig::new().enabled().first_line();
        let mut handler = HeaderHandler::new(config);

        // Empty lines are skipped
        assert!(!handler.process_first_line(b"").unwrap());
        assert!(!handler.process_first_line(b"").unwrap());
        // First non-empty line is header
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_lines_n_with_leading_empty() {
        // LinesN mode: empty lines are treated as data (not counted as header)
        let config = HeaderConfig::new().enabled().lines_n(2);
        let mut handler = HeaderHandler::new(config);

        // Empty lines are data
        assert!(!handler.process_first_line(b"").unwrap());
        assert!(!handler.process_first_line(b"").unwrap());
        // First two non-empty lines are header
        assert!(handler.process_first_line(b"# comment").unwrap());
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"# comment\ncol1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_lines_n_insufficient_lines() {
        // LinesN mode with insufficient lines - captures what's available
        let config = HeaderConfig::new().enabled().lines_n(3);
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"line1").unwrap());
        assert!(handler.process_first_line(b"line2").unwrap());
        // Only 2 lines available - both captured as header
        assert_eq!(handler.header(), Some(b"line1\nline2".as_slice()));
    }

    #[test]
    fn test_handler_hash_lines_no_hash() {
        // HashLines mode with no hash lines - no header
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // First non-hash line is data
        assert!(!handler.process_first_line(b"col1\tcol2").unwrap());
        assert!(handler.header().is_none());
    }

    #[test]
    fn test_handler_hash_lines1_only_hash() {
        // HashLines1 mode with only hash lines
        let config = HeaderConfig::new().enabled().hash_lines1();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"# comment only").unwrap());
        // No more lines - header is just the hash line
        assert_eq!(handler.header(), Some(b"# comment only".as_slice()));
    }

    #[test]
    fn test_handler_hash_lines_leading_empty() {
        // HashLines mode with leading empty lines - empty lines are data
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // Empty line is data
        assert!(!handler.process_first_line(b"").unwrap());
        assert!(!handler.process_first_line(b"").unwrap());
        // Hash line is header
        assert!(handler.process_first_line(b"# comment").unwrap());
        assert_eq!(handler.header(), Some(b"# comment".as_slice()));
    }

    #[test]
    fn test_handler_hash_lines1_mode() {
        let config = HeaderConfig::new().enabled().hash_lines1();
        let mut handler = HeaderHandler::new(config);

        // Hash lines are header
        assert!(handler.process_first_line(b"# comment").unwrap());
        // First non-hash line is also header (column names)
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        // Subsequent lines are data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());

        assert_eq!(handler.header(), Some(b"# comment\ncol1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_hash_lines1_no_hash() {
        let config = HeaderConfig::new().enabled().hash_lines1();
        let mut handler = HeaderHandler::new(config);

        // No hash lines, first line is column names
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());

        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_second_file_hash_lines() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // First file
        assert!(handler.process_first_line(b"# comment").unwrap());
        handler.end_of_file();

        // Second file - hash lines should be consumed but not captured
        assert!(handler.process_first_line(b"# another comment").unwrap());
        assert_eq!(handler.header(), Some(b"# comment".as_slice()));
    }

    #[test]
    fn test_handler_crlf_first_line() {
        // HeaderHandler receives lines without \r\n (already split by reader)
        // So we just test that it handles the line content correctly
        let config = HeaderConfig::new().enabled().first_line();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_single_line_no_newline() {
        // HeaderHandler works with single line (no newline needed)
        let config = HeaderConfig::new().enabled().first_line();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"col1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_lines_n_crlf_content() {
        // HeaderHandler receives lines without \r (already stripped by reader)
        let config = HeaderConfig::new().enabled().lines_n(2);
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"# comment").unwrap());
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        assert_eq!(handler.header(), Some(b"# comment\ncol1\tcol2".as_slice()));
    }

    #[test]
    fn test_handler_hash_lines_crlf_content() {
        // HeaderHandler receives lines without \r (already stripped by reader)
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        assert!(handler.process_first_line(b"# comment1").unwrap());
        assert!(handler.process_first_line(b"# comment2").unwrap());
        assert_eq!(handler.header(), Some(b"# comment1\n# comment2".as_slice()));

        // Non-hash line is data
        assert!(!handler.process_first_line(b"data").unwrap());
    }

    #[test]
    fn test_handler_lines_n_second_file() {
        let config = HeaderConfig::new().enabled().lines_n(2);
        let mut handler = HeaderHandler::new(config);

        // First file - capture header
        assert!(handler.process_first_line(b"line1").unwrap());
        handler.end_of_file();

        // Second file - should not treat as header
        assert!(!handler.process_first_line(b"data1").unwrap());
    }

    #[test]
    fn test_handler_hash_lines_second_file_non_hash() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // First file
        assert!(handler.process_first_line(b"# comment").unwrap());
        handler.end_of_file();

        // Second file - non-hash line should be data
        assert!(!handler.process_first_line(b"data").unwrap());
    }

    #[test]
    fn test_handler_hash_lines1_second_file() {
        let config = HeaderConfig::new().enabled().hash_lines1();
        let mut handler = HeaderHandler::new(config);

        // First file
        assert!(handler.process_first_line(b"# comment").unwrap());
        assert!(handler.process_first_line(b"col1\tcol2").unwrap());
        handler.end_of_file();

        // Second file - hash lines should be consumed but not captured
        assert!(handler.process_first_line(b"# another comment").unwrap());
        // Non-hash line is data
        assert!(!handler.process_first_line(b"val1\tval2").unwrap());
    }

    #[test]
    fn test_build_suffix_basic() {
        let items = vec!["count", "first", "last"];
        let suffix = build_suffix(&items, b'\t');
        assert_eq!(suffix, b"\tcount\tfirst\tlast");
    }

    #[test]
    fn test_build_suffix_empty() {
        let items: Vec<&str> = vec![];
        let suffix = build_suffix(&items, b'\t');
        assert!(suffix.is_empty());
    }

    #[test]
    fn test_build_suffix_single_item() {
        let items = vec!["n"];
        let suffix = build_suffix(&items, b'\t');
        assert_eq!(suffix, b"\tn");
    }

    #[test]
    fn test_build_suffix_custom_delimiter() {
        let items = vec!["a", "b"];
        let suffix = build_suffix(&items, b',');
        assert_eq!(suffix, b",a,b");
    }
}

/// Unified header structure that combines header metadata with field resolution capabilities.
///
/// This struct serves as the primary header representation for tva commands. It stores:
/// - All header lines (hash comments, LinesN lines, etc.)
/// - The column names line (for field name resolution)
/// - A cached index mapping for efficient field lookup
///
/// # Creating a Header
///
/// ## From raw column names
/// ```rust
/// use tva::libs::tsv::header::Header;
///
/// let header = Header::from_column_names(b"name\tage\tcity".to_vec(), '\t');
/// ```
///
/// ## From TsvReader output
/// `Header::from_info()` is used internally by commands that work with `TsvReader`.
/// The `HeaderInfo` struct is created by the reader and passed to commands.
///
/// # Field Resolution
///
/// The header maintains a cache for efficient field name lookup:
/// ```rust
/// use tva::libs::tsv::header::Header;
///
/// let header = Header::from_column_names(b"a\tb\tc".to_vec(), '\t');
///
/// // 0-based indexing
/// assert_eq!(header.get_index("a"), Some(0));
/// assert_eq!(header.get_index("b"), Some(1));
/// assert_eq!(header.get_index("z"), None); // Not found
/// ```
#[derive(Clone, Debug)]
pub struct Header {
    /// All header lines (e.g., hash comment lines, or first N lines in LinesN mode).
    /// These are written before the column names line.
    pub lines: Vec<Vec<u8>>,
    /// The specific line containing column names (if applicable).
    /// This is used for field name resolution.
    pub column_names: Option<Vec<u8>>,
    /// Field delimiter used for parsing column names.
    delimiter: char,
    /// Cache for field name to index lookup.
    /// Maps field name -> 0-based column index.
    index_cache: Option<HashMap<String, usize>>,
}

impl Header {
    /// Creates a new Header from HeaderInfo.
    ///
    /// # Arguments
    /// * `info` - The HeaderInfo returned by TsvReader
    /// * `delimiter` - Field delimiter character (typically '\t')
    pub fn from_info(info: HeaderInfo, delimiter: char) -> Self {
        let index_cache = info.column_names_line.as_ref().and_then(|bytes| {
            if bytes.is_empty() {
                // Empty column names line means 0 columns
                Some(HashMap::new())
            } else {
                let line = String::from_utf8_lossy(bytes);
                Some(
                    line.split(delimiter)
                        .enumerate()
                        .map(|(idx, name)| (name.to_string(), idx))
                        .collect(),
                )
            }
        });

        Self {
            lines: info.lines,
            column_names: info.column_names_line,
            delimiter,
            index_cache,
        }
    }

    /// Creates a Header from raw column names bytes.
    ///
    /// This is useful when you only have column names without other header lines.
    pub fn from_column_names(column_names: Vec<u8>, delimiter: char) -> Self {
        let index_cache = if column_names.is_empty() {
            // Empty column names means 0 columns
            Some(HashMap::new())
        } else {
            let line = String::from_utf8_lossy(&column_names);
            Some(
                line.split(delimiter)
                    .enumerate()
                    .map(|(idx, name)| (name.to_string(), idx))
                    .collect(),
            )
        };

        Self {
            lines: Vec::new(),
            column_names: Some(column_names),
            delimiter,
            index_cache,
        }
    }

    /// Returns true if this header has column names available.
    pub fn has_column_names(&self) -> bool {
        self.column_names.is_some()
    }

    /// Gets the index of a field by name (0-based).
    ///
    /// Returns `Some(index)` if found, `None` otherwise.
    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.index_cache.as_ref()?.get(name).copied()
    }

    /// Returns the column names as a vector of strings.
    ///
    /// Returns `None` if no column names are available.
    pub fn column_names_list(&self) -> Option<Vec<String>> {
        self.column_names.as_ref().map(|bytes| {
            if bytes.is_empty() {
                Vec::new()
            } else {
                String::from_utf8_lossy(bytes)
                    .split(self.delimiter)
                    .map(|s| s.to_string())
                    .collect()
            }
        })
    }

    /// Returns the number of columns.
    ///
    /// Returns `None` if no column names are available.
    pub fn column_count(&self) -> Option<usize> {
        self.index_cache.as_ref().map(|cache| cache.len())
    }
}

/// Writes header to output in standard format.
///
/// Writes all header lines followed by the column names line (if present).
/// An optional suffix can be appended to the column names line (e.g., for equiv mode).
///
/// # Arguments
/// * `writer` - Output writer
/// * `header` - Header to write
/// * `suffix` - Optional suffix to append to column names line
///
/// # Example
/// ```
/// use tva::libs::tsv::header::{Header, write_header};
///
/// let header = Header::from_column_names(b"col1\tcol2".to_vec(), '\t');
/// let mut output = Vec::new();
/// write_header(&mut output, &header, None).unwrap();
/// assert_eq!(output, b"col1\tcol2\n");
/// ```
pub fn write_header<W: Write>(
    writer: &mut W,
    header: &Header,
    suffix: Option<&[u8]>,
) -> io::Result<()> {
    // Write all header lines (hash lines, LinesN lines, etc.)
    for line in &header.lines {
        writer.write_all(line)?;
        writer.write_all(b"\n")?;
    }

    // Write column names line with optional suffix
    if let Some(ref column_names) = header.column_names {
        writer.write_all(column_names)?;
        if let Some(s) = suffix {
            writer.write_all(s)?;
        }
        writer.write_all(b"\n")?;
    }

    Ok(())
}

/// Builds a suffix string from items for appending to column names.
///
/// This is useful for commands like `uniq --equiv` that add extra columns to the header.
///
/// # Arguments
/// * `items` - Items to include in the suffix
/// * `delimiter` - Delimiter to use between items
///
/// # Returns
/// A byte vector containing the suffix (starts with delimiter).
pub fn build_suffix(items: &[impl AsRef<str>], delimiter: u8) -> Vec<u8> {
    let mut result = Vec::new();
    for item in items {
        result.push(delimiter);
        result.extend_from_slice(item.as_ref().as_bytes());
    }
    result
}
