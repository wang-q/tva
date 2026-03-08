//! Header detection and handling for TSV data.
//!
//! Provides unified header processing logic for all tva commands.

/// Configuration for header detection.
#[derive(Debug, Clone)]
pub struct HeaderConfig {
    /// Whether header processing is enabled.
    pub enabled: bool,
    /// Number of lines to treat as header (0 = auto-detect).
    pub lines: usize,
    /// Whether to include comment lines (starting with '#') as part of header.
    pub include_comments: bool,
}

impl HeaderConfig {
    /// Creates a new header config with default settings (disabled).
    pub fn new() -> Self {
        Self {
            enabled: false,
            lines: 1,
            include_comments: false,
        }
    }

    /// Enables header processing.
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Sets the number of header lines.
    pub fn lines(mut self, n: usize) -> Self {
        self.lines = n;
        self
    }

    /// Enables including comment lines as header.
    /// Also sets lines to 0 (auto-detect) to allow collecting all comment lines.
    pub fn include_comments(mut self) -> Self {
        self.include_comments = true;
        self.lines = 0; // Auto-detect mode when including comments
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

    let mut lines = Vec::new();
    let mut pos = 0;
    let mut line_count = 0;

    // If fixed lines specified, take exactly that many non-empty lines
    if config.lines > 0 {
        // First, skip any leading empty lines
        while pos < data.len() {
            let line_end = find_line_end(data, pos);
            let line = &data[pos..line_end];
            
            // Remove trailing '\r' for Windows line endings
            let line = if line.ends_with(b"\r") {
                &line[..line.len() - 1]
            } else {
                line
            };
            
            if !line.is_empty() {
                break;
            }
            
            // Move past the newline
            pos = line_end;
            if pos < data.len() && data[pos] == b'\n' {
                pos += 1;
            }
        }
        
        // Now collect exactly config.lines non-empty lines
        while line_count < config.lines && pos < data.len() {
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
        
        return DetectedHeader {
            lines,
            bytes_consumed: pos,
        };
    }

    // Auto-detect: skip empty lines, then collect header
    loop {
        if pos >= data.len() {
            break;
        }

        let line_end = find_line_end(data, pos);
        let line = &data[pos..line_end];
        
        // Remove trailing '\r'
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

        // Skip empty lines at the beginning
        if line.is_empty() && lines.is_empty() {
            pos = next_pos;
            continue;
        }

        // Check if it's a comment line
        let is_comment = line.starts_with(b"#");

        if is_comment {
            if config.include_comments {
                // Collect comment lines as part of header
                lines.push(line.to_vec());
                pos = next_pos;
                // Continue to collect more comments or the actual header
            } else {
                // Comment but include_comments is false - this is data
                break;
            }
        } else {
            // First non-empty, non-comment line is the header
            lines.push(line.to_vec());
            pos = next_pos;
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
}

impl HeaderHandler {
    /// Creates a new header handler with the given config.
    pub fn new(config: HeaderConfig) -> Self {
        Self {
            config,
            captured_header: None,
            is_first_file: true,
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
        // Skip empty lines - they're not header
        if line.is_empty() {
            return Ok(false);
        }

        // Check if it's a comment line
        let is_comment = line.starts_with(b"#");

        if !self.config.enabled {
            return Ok(false);
        }

        // If include_comments is enabled, comment lines are part of header
        if is_comment && self.config.include_comments {
            if self.is_first_file {
                // Capture comment line
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
            return Ok(true);
        }

        // If it's a comment but include_comments is disabled, treat as data
        if is_comment {
            return Ok(false);
        }

        // First non-comment line
        if self.is_first_file {
            self.captured_header = Some(line.to_vec());
            self.is_first_file = false;
            return Ok(true);
        }

        Ok(false)
    }

    /// Marks the end of the first file.
    pub fn end_of_file(&mut self) {
        self.is_first_file = false;
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
    fn test_header_fixed_lines() {
        let config = HeaderConfig::new().enabled().lines(2);
        let data = b"# comment\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0], b"# comment");
        assert_eq!(result.lines[1], b"col1\tcol2");
    }

    #[test]
    fn test_header_with_comments() {
        let config = HeaderConfig::new().enabled().include_comments();
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
    fn test_header_skip_empty_lines() {
        let config = HeaderConfig::new().enabled();
        let data = b"\n\ncol1\tcol2\nval1\tval2\n";
        let result = detect_header(data, &config);
        assert!(result.has_header());
        assert_eq!(result.lines[0], b"col1\tcol2");
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
}
