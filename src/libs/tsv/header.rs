//! Header detection and handling for TSV data.
//!
//! Provides unified header processing logic for all tva commands.

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

/// Result of header detection.
#[derive(Debug)]
pub struct DetectedHeader {
    /// The detected header lines (may be empty if no header).
    pub lines: Vec<Vec<u8>>,
    /// Number of bytes consumed from the input.
    pub bytes_consumed: usize,
}

impl DetectedHeader {
    /// Returns true if any header lines were detected.
    pub fn has_header(&self) -> bool {
        !self.lines.is_empty()
    }

    /// Returns the header as a single byte slice (lines joined with '\n').
    pub fn as_bytes(&self) -> Vec<u8> {
        if self.lines.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                result.push(b'\n');
            }
            result.extend_from_slice(line);
        }
        result
    }
}

/// Detects header from raw bytes according to the config.
///
/// Returns the detected header lines and the number of bytes consumed.
/// The caller can skip `bytes_consumed` to get to the data section.
pub fn detect_header(data: &[u8], config: &HeaderConfig) -> DetectedHeader {
    if !config.enabled {
        return DetectedHeader {
            lines: Vec::new(),
            bytes_consumed: 0,
        };
    }

    match config.mode {
        HeaderMode::FirstLine => detect_first_line_header(data),
        HeaderMode::LinesN(n) => detect_lines_n_header(data, n),
        HeaderMode::HashLines => detect_hash_lines_header(data, false),
        HeaderMode::HashLines1 => detect_hash_lines_header(data, true),
    }
}

/// Detects header using FirstLine mode: first line is the header, even if empty.
fn detect_first_line_header(data: &[u8]) -> DetectedHeader {
    let mut lines = Vec::new();
    let mut pos = 0;

    if pos < data.len() {
        let line_end = find_line_end(data, pos);
        let line = &data[pos..line_end];

        // Remove trailing '\r' for Windows line endings
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };

        // Move past this line for next iteration
        pos = if line_end < data.len() && data[line_end] == b'\n' {
            line_end + 1
        } else {
            line_end
        };

        // First line is the header (even if empty)
        lines.push(line.to_vec());
    }

    DetectedHeader {
        lines,
        bytes_consumed: pos,
    }
}

/// Detects header using LinesN mode: exactly N lines are the header (including empty lines).
fn detect_lines_n_header(data: &[u8], n: usize) -> DetectedHeader {
    let mut lines = Vec::new();
    let mut pos = 0;
    let mut line_count = 0;

    // Collect exactly n lines (including empty lines)
    while line_count < n && pos < data.len() {
        let line_end = find_line_end(data, pos);
        let line = &data[pos..line_end];

        // Remove trailing '\r' for Windows line endings
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };

        lines.push(line.to_vec());
        line_count += 1;

        // Move past the newline
        pos = line_end;
        if pos < data.len() && data[pos] == b'\n' {
            pos += 1;
        }
    }

    DetectedHeader {
        lines,
        bytes_consumed: pos,
    }
}

/// Detects header using HashLines or HashLinesPlusOne mode.
///
/// When `include_next_line` is false: only consecutive '#' lines are the header.
/// When `include_next_line` is true: consecutive '#' lines + the next line are the header.
fn detect_hash_lines_header(data: &[u8], include_next_line: bool) -> DetectedHeader {
    let mut lines = Vec::new();
    let mut pos = 0;

    loop {
        if pos >= data.len() {
            break;
        }

        let line_end = find_line_end(data, pos);
        let line = &data[pos..line_end];

        // Remove trailing '\r' for Windows line endings
        let line = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };

        // Move past this line for next iteration
        let next_pos = if line_end < data.len() && data[line_end] == b'\n' {
            line_end + 1
        } else {
            line_end
        };

        // Skip leading empty lines - they're not part of hash-based header
        if line.is_empty() && lines.is_empty() {
            pos = next_pos;
            continue;
        }

        // Check if it's a hash line (starts with '#')
        let is_hash = line.starts_with(b"#");

        if is_hash {
            // Collect hash lines as part of header
            lines.push(line.to_vec());
            pos = next_pos;
            // Continue to collect more hash lines
        } else if include_next_line {
            // First non-hash line is also part of header (column names)
            lines.push(line.to_vec());
            pos = next_pos;
            break;
        } else {
            // Not a hash line and we don't need to collect it
            break;
        }
    }

    DetectedHeader {
        lines,
        bytes_consumed: pos,
    }
}

/// Finds the end of the current line (position of '\n' or end of data).
fn find_line_end(data: &[u8], start: usize) -> usize {
    for i in start..data.len() {
        if data[i] == b'\n' {
            return i;
        }
    }
    data.len()
}

/// Header handler for streaming processing.
///
/// This is useful when processing TSV data line by line, where you need to
/// capture the header from the first input file and potentially write it to output.
pub struct HeaderHandler {
    config: HeaderConfig,
    captured_header: Option<Vec<u8>>,
    is_first_file: bool,
    /// For LinesN mode: remaining lines to collect as header
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
    fn test_header_disabled() {
        let config = HeaderConfig::new();
        let data = b"col1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(!result.has_header());
        assert_eq!(result.bytes_consumed, 0);
    }

    #[test]
    fn test_header_lines_n() {
        let config = HeaderConfig::new().enabled().lines_n(2);
        let data = b"# comment\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0], b"# comment");
        assert_eq!(result.lines[1], b"col1\tcol2");
    }

    #[test]
    fn test_header_with_hash_lines() {
        // HashLines mode: only '#' lines are header
        let config = HeaderConfig::new().enabled().hash_lines();
        let data = b"# comment 1\n# comment 2\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0], b"# comment 1");
        assert_eq!(result.lines[1], b"# comment 2");
    }

    #[test]
    fn test_header_with_hash_lines1() {
        // HashLines1 mode: '#' lines + next line are header (for column names)
        let config = HeaderConfig::new().enabled().hash_lines1();
        let data = b"# comment 1\n# comment 2\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.lines[0], b"# comment 1");
        assert_eq!(result.lines[1], b"# comment 2");
        assert_eq!(result.lines[2], b"col1\tcol2");
    }

    #[test]
    fn test_header_auto_detect() {
        let config = HeaderConfig::new().enabled();
        let data = b"col1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0], b"col1\tcol2");
    }

    #[test]
    fn test_header_first_line_with_empty_lines() {
        // FirstLine mode now takes the first line even if empty
        let config = HeaderConfig::new().enabled();
        let data = b"\n\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        // First line is empty
        assert_eq!(result.lines[0], b"");
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
    fn test_detected_header_as_bytes_single() {
        let header = DetectedHeader {
            lines: vec![b"col1\tcol2".to_vec()],
            bytes_consumed: 10,
        };
        assert_eq!(header.as_bytes(), b"col1\tcol2");
    }

    #[test]
    fn test_detected_header_as_bytes_multiple() {
        let header = DetectedHeader {
            lines: vec![b"# comment".to_vec(), b"col1\tcol2".to_vec()],
            bytes_consumed: 20,
        };
        assert_eq!(header.as_bytes(), b"# comment\ncol1\tcol2");
    }

    #[test]
    fn test_detected_header_as_bytes_empty() {
        let header = DetectedHeader {
            lines: vec![],
            bytes_consumed: 0,
        };
        assert!(header.as_bytes().is_empty());
    }

    #[test]
    fn test_first_line_header_empty_data() {
        let config = HeaderConfig::new().enabled().first_line();
        let data = b"";
        let result = detect_header(data, &config);
        assert!(!result.has_header());
        assert_eq!(result.bytes_consumed, 0);
    }

    #[test]
    fn test_first_line_header_only_empty_lines() {
        // FirstLine mode takes the first line even if empty
        let config = HeaderConfig::new().enabled().first_line();
        let data = b"\n\n\n";
        let result = detect_header(data, &config);
        // Now we have a header (the first empty line)
        assert!(result.has_header());
        assert_eq!(result.lines[0], b""); // First line is empty
        assert_eq!(result.bytes_consumed, 1); // Only consumed first line
    }

    #[test]
    fn test_lines_n_header_with_leading_empty_lines() {
        // LinesN mode now takes the first N lines (including empty lines)
        let config = HeaderConfig::new().enabled().lines_n(2);
        let data = b"\n\n# comment\ncol1\tcol2\ndata\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        // First two lines are empty
        assert_eq!(result.lines[0], b"");
        assert_eq!(result.lines[1], b"");
    }

    #[test]
    fn test_lines_n_header_insufficient_lines() {
        let config = HeaderConfig::new().enabled().lines_n(3);
        let data = b"line1\nline2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2); // Only 2 lines available
    }

    #[test]
    fn test_hash_lines_no_hash_found() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let data = b"col1\tcol2\ndata\n";
        let result = detect_header(data, &config);
        assert!(!result.has_header());
        assert_eq!(result.bytes_consumed, 0);
    }

    #[test]
    fn test_hash_lines1_no_data_after_hash() {
        let config = HeaderConfig::new().enabled().hash_lines1();
        let data = b"# comment only\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0], b"# comment only");
    }

    #[test]
    fn test_hash_lines_with_leading_empty() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let data = b"\n\n# comment\ndata\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0], b"# comment");
    }

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
    fn test_handler_empty_line() {
        let config = HeaderConfig::new().enabled();
        let mut handler = HeaderHandler::new(config);

        // Empty lines should not be treated as header
        assert!(!handler.process_first_line(b"").unwrap());
        assert!(handler.header().is_none());
    }

    #[test]
    fn test_handler_hash_lines_mode() {
        let config = HeaderConfig::new().enabled().hash_lines();
        let mut handler = HeaderHandler::new(config);

        // First hash line
        assert!(handler.process_first_line(b"# comment 1").unwrap());
        // Second hash line
        assert!(handler.process_first_line(b"# comment 2").unwrap());
        // Non-hash line is data
        assert!(!handler.process_first_line(b"col1\tcol2").unwrap());

        assert!(handler.should_output_header());
        assert_eq!(
            handler.header(),
            Some(b"# comment 1\n# comment 2".as_slice())
        );
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
    fn test_detect_header_crlf() {
        let config = HeaderConfig::new().enabled().first_line();
        let data = b"col1\tcol2\r\nval1\tval2\r\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines[0], b"col1\tcol2"); // \r removed
    }

    #[test]
    fn test_detect_header_no_final_newline() {
        let config = HeaderConfig::new().enabled().first_line();
        let data = b"col1\tcol2"; // No newline at end
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines[0], b"col1\tcol2");
    }

    #[test]
    fn test_lines_n_with_crlf() {
        // Test Windows line endings (\r\n) in LinesN mode
        let config = HeaderConfig::new().enabled().lines_n(2);
        let data = b"# comment\r\ncol1\tcol2\r\ndata\r\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0], b"# comment"); // \r removed
        assert_eq!(result.lines[1], b"col1\tcol2"); // \r removed
    }

    #[test]
    fn test_hash_lines_with_crlf() {
        // Test Windows line endings (\r\n) in HashLines mode
        let config = HeaderConfig::new().enabled().hash_lines();
        let data = b"# comment1\r\n# comment2\r\ndata\r\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0], b"# comment1"); // \r removed
        assert_eq!(result.lines[1], b"# comment2"); // \r removed
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
}
