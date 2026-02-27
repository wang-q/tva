//! High-performance, zero-copy TSV record.
//!
//! Designed to mimic `csv::ByteRecord` but specialized for TSV (no quotes, fixed delimiter).

use std::fmt;

/// A trait for abstracting over different row representations (e.g., owned TsvRecord, borrowed TsvRow, or split string slices).
///
/// This allows functions (like filter evaluation) to work on any data structure that provides field access.
pub trait Row {
    /// Get the bytes of the field at 1-based index `idx`.
    /// Returns `None` if `idx` is 0 or out of bounds.
    fn get_bytes(&self, idx: usize) -> Option<&[u8]>;

    /// Get the string content of the field at 1-based index `idx`.
    /// Returns `None` if `idx` is 0, out of bounds, or if the field is not valid UTF-8.
    fn get_str(&self, idx: usize) -> Option<&str> {
        let b = self.get_bytes(idx)?;
        std::str::from_utf8(b).ok()
    }
}

/// A lightweight, zero-copy view into a TSV row.
///
/// This struct holds references to the line data and an array of field end positions (delimiters).
/// It does not own the data.
pub struct TsvRow<'a, 'b> {
    pub line: &'a [u8],
    pub ends: &'b [usize],
}

impl<'a, 'b> Row for TsvRow<'a, 'b> {
    fn get_bytes(&self, idx: usize) -> Option<&[u8]> {
        if idx == 0 {
            return None;
        }
        if self.line.is_empty() {
            return if idx == 1 { Some(&[]) } else { None };
        }

        let i = idx - 1;
        if i > self.ends.len() {
            return None;
        }

        let start = if idx == 1 { 0 } else { self.ends[idx - 2] + 1 };
        let end = if i < self.ends.len() {
            self.ends[i]
        } else {
            self.line.len()
        };
        Some(&self.line[start..end])
    }
}

/// A wrapper for `&[&str]` to implement `Row` trait.
/// This is useful for tests or when working with already split strings.
pub struct StrSliceRow<'a> {
    pub fields: &'a [&'a str],
}

impl<'a> Row for StrSliceRow<'a> {
    fn get_bytes(&self, idx: usize) -> Option<&[u8]> {
        if idx == 0 {
            return None;
        }
        self.fields.get(idx - 1).map(|s| s.as_bytes())
    }

    fn get_str(&self, idx: usize) -> Option<&str> {
        if idx == 0 {
            return None;
        }
        self.fields.get(idx - 1).copied()
    }
}

/// A single TSV record stored as raw bytes.
///
/// Unlike `csv::ByteRecord`, this struct assumes TSV format (no escaping, no quotes).
/// It stores the entire line contiguously and a list of field end positions.
#[derive(Clone, Eq)]
pub struct TsvRecord {
    /// All fields in this record, stored contiguously.
    /// For TSV, this is just the raw line (excluding the newline).
    line: Vec<u8>,
    /// The ending index of each field in `line`.
    /// E.g., for "a\tb\tc", line is "a\tb\tc", ends are [1, 3, 5].
    /// Field 0: 0..1 ("a")
    /// Field 1: 2..3 ("b")
    /// Field 2: 4..5 ("c")
    ends: Vec<usize>,
}

impl PartialEq for TsvRecord {
    fn eq(&self, other: &TsvRecord) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl fmt::Debug for TsvRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TsvRecord(")?;
        f.debug_list()
            .entries(self.iter().map(|b| String::from_utf8_lossy(b)))
            .finish()?;
        write!(f, ")")?;
        Ok(())
    }
}

impl Default for TsvRecord {
    #[inline]
    fn default() -> TsvRecord {
        TsvRecord::new()
    }
}

impl TsvRecord {
    /// Create a new empty `TsvRecord`.
    #[inline]
    pub fn new() -> TsvRecord {
        TsvRecord {
            line: Vec::new(),
            ends: Vec::new(),
        }
    }

    /// Create a new empty `TsvRecord` with capacity.
    #[inline]
    pub fn with_capacity(line_cap: usize, fields_cap: usize) -> TsvRecord {
        TsvRecord {
            line: Vec::with_capacity(line_cap),
            ends: Vec::with_capacity(fields_cap),
        }
    }

    /// Clear the record.
    #[inline]
    pub fn clear(&mut self) {
        self.line.clear();
        self.ends.clear();
    }

    /// Parse a line into this record.
    ///
    /// This replaces the current content.
    /// The line should NOT contain the trailing newline.
    #[inline]
    pub fn parse_line(&mut self, line: &[u8], delimiter: u8) {
        self.clear();
        self.line.extend_from_slice(line);

        // Scan for delimiters
        // Using memchr_iter for speed
        for pos in memchr::memchr_iter(delimiter, &self.line) {
            self.ends.push(pos);
        }
        // Push the end of the last field
        self.ends.push(self.line.len());
    }

    /// Returns the number of fields.
    #[inline]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// Returns true if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the field at index `i`.
    #[inline]
    pub fn get(&self, i: usize) -> Option<&[u8]> {
        if i >= self.ends.len() {
            return None;
        }
        let end = self.ends[i];
        let start = if i == 0 {
            0
        } else {
            self.ends[i - 1] + 1
        };
        Some(&self.line[start..end])
    }

    /// Iterator over fields.
    #[inline]
    pub fn iter(&self) -> TsvRecordIter<'_> {
        TsvRecordIter {
            record: self,
            pos: 0,
        }
    }

    /// Access the underlying line buffer directly.
    #[inline]
    pub fn as_line(&self) -> &[u8] {
        &self.line
    }
}

impl Row for TsvRecord {
    fn get_bytes(&self, idx: usize) -> Option<&[u8]> {
        if idx == 0 {
            return None;
        }
        self.get(idx - 1)
    }
}

pub struct TsvRecordIter<'a> {
    record: &'a TsvRecord,
    pos: usize,
}

impl<'a> Iterator for TsvRecordIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.record.len() {
            let item = self.record.get(self.pos);
            self.pos += 1;
            item
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let mut rec = TsvRecord::new();
        rec.parse_line(b"a\tb\tc", b'\t');
        assert_eq!(rec.len(), 3);
        assert_eq!(rec.get(0), Some(b"a".as_slice()));
        assert_eq!(rec.get(1), Some(b"b".as_slice()));
        assert_eq!(rec.get(2), Some(b"c".as_slice()));
    }

    #[test]
    fn test_parse_empty_fields() {
        let mut rec = TsvRecord::new();
        rec.parse_line(b"\t\t", b'\t');
        assert_eq!(rec.len(), 3);
        assert_eq!(rec.get(0), Some(b"".as_slice()));
        assert_eq!(rec.get(1), Some(b"".as_slice()));
        assert_eq!(rec.get(2), Some(b"".as_slice()));
    }
}
