//! High-performance parsing utilities using `lexical` crate.
//!
//! This module provides a central place for parsing logic, ensuring consistency
//! and performance across all commands (stats, filter, sampling).

/// Trims ASCII whitespace from the beginning and end of a byte slice.
#[inline]
fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &bytes[start..end]
}

/// Parses a byte slice into an f64 using lexical's high-performance algorithm.
/// Returns None if parsing fails.
///
/// This handles trimming of whitespace automatically.
/// This is significantly faster than `str::parse::<f64>` and operates directly on bytes,
/// avoiding UTF-8 validation overhead.
#[inline]
pub fn fast_parse_f64(bytes: &[u8]) -> Option<f64> {
    let trimmed = trim_bytes(bytes);
    if trimmed.is_empty() {
        return None;
    }
    lexical::parse(trimmed).ok()
}
