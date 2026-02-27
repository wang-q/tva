//! High-performance TSV splitting utilities using SIMD.
//!
//! This module provides an iterator that yields byte slices for each field in a TSV line.
//! It uses `memchr` (which uses SIMD internally) to quickly find tab delimiters.

use memchr::memchr_iter;

/// An iterator over fields in a TSV line.
///
/// This iterator yields `&[u8]` slices for each field. It is designed to be zero-allocation.
/// It assumes the delimiter is a tab character (`\t`).
pub struct TsvSplitter<'a> {
    data: &'a [u8],
    // Iterator over delimiter positions.
    // We use a specific iterator from memchr for max performance.
    iter: memchr::Memchr<'a>,
    last_pos: usize,
    finished: bool,
}

impl<'a> TsvSplitter<'a> {
    /// Create a new splitter for the given byte slice.
    #[inline]
    pub fn new(data: &'a [u8], delimiter: u8) -> Self {
        Self {
            data,
            iter: memchr_iter(delimiter, data),
            last_pos: 0,
            finished: false,
        }
    }
}

impl<'a> Iterator for TsvSplitter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.iter.next() {
            Some(pos) => {
                // Yield the slice from last_pos to pos (exclusive)
                // SAFETY: memchr returns valid indices within slice
                let field = unsafe { self.data.get_unchecked(self.last_pos..pos) };
                self.last_pos = pos + 1; // Skip the delimiter
                Some(field)
            }
            None => {
                self.finished = true;
                // SAFETY: last_pos is guaranteed <= data.len()
                Some(unsafe { self.data.get_unchecked(self.last_pos..) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_basic() {
        let line = b"a\tb\tc";
        let fields: Vec<&[u8]> = TsvSplitter::new(line, b'\t').collect();
        assert_eq!(fields, vec![&b"a"[..], &b"b"[..], &b"c"[..]]);
    }

    #[test]
    fn test_split_empty_fields() {
        let line = b"\t\t";
        let fields: Vec<&[u8]> = TsvSplitter::new(line, b'\t').collect();
        assert_eq!(fields, vec![&b""[..], &b""[..], &b""[..]]);
    }

    #[test]
    fn test_split_trailing_empty() {
        let line = b"a\t";
        let fields: Vec<&[u8]> = TsvSplitter::new(line, b'\t').collect();
        assert_eq!(fields, vec![&b"a"[..], &b""[..]]);
    }

    #[test]
    fn test_split_single_field() {
        let line = b"abc";
        let fields: Vec<&[u8]> = TsvSplitter::new(line, b'\t').collect();
        assert_eq!(fields, vec![&b"abc"[..]]);
    }

    #[test]
    fn test_split_custom_delimiter() {
        let line = b"a,b,c";
        let fields: Vec<&[u8]> = TsvSplitter::new(line, b',').collect();
        assert_eq!(fields, vec![&b"a"[..], &b"b"[..], &b"c"[..]]);
    }
}
