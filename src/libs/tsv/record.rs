//! High-performance, zero-copy TSV record.
//!
//! Designed to mimic `csv::ByteRecord` but specialized for TSV (no quotes, fixed delimiter).

use std::fmt;

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
            // The previous field ended at ends[i-1].
            // That position was the delimiter (or end of line if we handle it carefully).
            // Wait, my `ends` logic stores the position of the delimiter.
            // "a\tb" -> ends: [1, 3]
            // Field 0: 0..1 ("a")
            // Field 1: 2..3 ("b")
            // Delimiter is at 1. Start of field 1 is 1 + 1 = 2.
            self.ends[i - 1] + 1
        };

        // Safety check: start <= end
        // end is either delimiter pos or len.
        // start is prev_delim + 1.
        // if "a\t\tb", ends: [1, 2, 4].
        // i=1: prev=1. start=2. end=2. -> "" (empty field). Correct.
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
