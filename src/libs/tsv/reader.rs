//! High-performance, zero-copy TSV reader.
//!
//! This module provides `TsvReader`, which manages an internal buffer to allow
//! iterating over TSV records with minimal allocation.

use crate::libs::tsv::record::TsvRow;
use std::io::{self, Read, Write};

/// A reader that efficiently scans for TSV records (lines) in a byte stream.
pub struct TsvReader<R> {
    reader: R,
    /// Internal buffer for reading data.
    buf: Vec<u8>,
    /// The number of valid bytes in `buf`.
    len: usize,
    /// The current read position in `buf`.
    pos: usize,
    /// Whether we've reached EOF on the underlying reader.
    eof: bool,
}

impl<R: Read> TsvReader<R> {
    /// Create a new `TsvReader` with default buffer size (64KB).
    pub fn new(reader: R) -> Self {
        // Use a larger buffer (64KB) for better I/O throughput.
        // In benchmarks, 8KB was good, but 64KB is standard for bulk reads.
        Self::with_capacity(reader, 64 * 1024)
    }

    /// Create a new `TsvReader` with specified buffer capacity.
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        Self {
            reader,
            buf: vec![0; capacity],
            len: 0,
            pos: 0,
            eof: false,
        }
    }

    /// Reads the first record as a header and returns it as a `Vec<u8>`.
    ///
    /// This advances the reader position. It should be called before `for_each_record`
    /// if header processing is needed.
    ///
    /// Note: This method copies the header data because the internal buffer
    /// will be reused for subsequent records.
    pub fn read_header(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut header = None;
        // We use a temporary closure to capture the first record
        let res = self.for_each_record(|record| {
            header = Some(record.to_vec());
            Err(io::Error::new(io::ErrorKind::Interrupted, "Stop iteration"))
        });

        match res {
            Ok(_) => Ok(None), // Empty file
            Err(e) if e.kind() == io::ErrorKind::Interrupted => Ok(header),
            Err(e) => Err(e),
        }
    }

    /// Copies the remaining data (buffered and unread) to the given writer.
    ///
    /// This consumes the rest of the stream efficiently without line splitting.
    /// This is useful when you want to pipe the rest of the file directly after processing headers.
    pub fn copy_remainder_to<W: Write>(&mut self, writer: &mut W) -> io::Result<u64> {
        let mut total_copied = 0;

        // 1. Write remaining buffered data
        if self.pos < self.len {
            let buffered_data = &self.buf[self.pos..self.len];
            writer.write_all(buffered_data)?;
            total_copied += buffered_data.len() as u64;
            self.pos = self.len; // Mark buffer as consumed
        }

        // 2. Copy the rest from the underlying reader
        // Note: io::copy handles reading until EOF
        let copied = io::copy(&mut self.reader, writer)?;
        total_copied += copied;
        self.eof = true;

        Ok(total_copied)
    }

    /// Iterate over records using a closure.
    ///
    /// The closure receives a `&[u8]` slice representing the record (excluding the newline).
    /// This avoids allocating a `String` or `Vec<u8>` for each record.
    ///
    /// # Errors
    /// Returns any I/O error encountered while reading.
    pub fn for_each_record<F>(&mut self, mut func: F) -> io::Result<()>
    where
        F: FnMut(&[u8]) -> io::Result<()>,
    {
        loop {
            // Calculate available data from current position
            let available = &self.buf[self.pos..self.len];

            // Search for newline in available data
            match memchr::memchr(b'\n', available) {
                Some(idx) => {
                    // Found a newline at `self.pos + idx`
                    let record_end = self.pos + idx;

                    // Handle potential CR before LF
                    let mut content_end = record_end;
                    if idx > 0 && self.buf[record_end - 1] == b'\r' {
                        content_end -= 1;
                    }

                    let record = &self.buf[self.pos..content_end];
                    // Call the function
                    // If function returns Interrupted, we stop but don't propagate error (if caller handles it)
                    match func(record) {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                            // Advance position past the newline so next call starts correctly
                            self.pos = record_end + 1;
                            return Err(e);
                        }
                        Err(e) => return Err(e),
                    }

                    // Advance position past the newline
                    self.pos = record_end + 1;
                }
                None => {
                    // No newline found in current window.
                    // We need to read more data.

                    // If we have processed everything in the buffer, reset pos and len
                    if self.pos >= self.len {
                        self.pos = 0;
                        self.len = 0;
                    } else if self.pos > 0 {
                        // Move leftover data to the beginning of the buffer
                        // Use copy_within (memmove)
                        self.buf.copy_within(self.pos..self.len, 0);
                        self.len -= self.pos;
                        self.pos = 0;
                    }

                    // Check if we need to grow the buffer
                    // If buffer is full (len == capacity) and pos is 0, it means we have a record larger than buffer.
                    if self.len == self.buf.len() {
                        self.buf.resize(self.buf.len() * 2, 0);
                    }

                    // Read more data into the free space
                    let read_len = self.reader.read(&mut self.buf[self.len..])?;
                    if read_len == 0 {
                        // EOF reached.
                        // If we have leftover data, yield it as the last record (if not empty)
                        if self.len > 0 {
                            let mut content_end = self.len;
                            if self.buf[content_end - 1] == b'\r' {
                                content_end -= 1;
                            }
                            let record = &self.buf[0..content_end];
                            func(record)?;
                            self.len = 0;
                        }
                        self.eof = true;
                        return Ok(());
                    }
                    self.len += read_len;
                }
            }
        }
    }

    /// Iterate over rows (parsed records) using a closure.
    ///
    /// This is a convenience wrapper around `for_each_record` that constructs a `TsvRow`
    /// for each record.
    ///
    /// The delimiter parameter specifies the field separator (default is TAB).
    pub fn for_each_row<F>(&mut self, delimiter: u8, mut func: F) -> io::Result<()>
    where
        F: FnMut(&TsvRow) -> io::Result<()>,
    {
        let mut ends = Vec::new();
        self.for_each_record(|record| {
            ends.clear();
            // Pre-calculate field delimiters for the row
            for pos in memchr::memchr_iter(delimiter, record) {
                ends.push(pos);
            }
            let row = TsvRow {
                line: record,
                ends: &ends,
            };
            func(&row)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_basic() {
        let data = b"a\tb\nc\td\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);
        let mut records = Vec::new();

        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], b"a\tb");
        assert_eq!(records[1], b"c\td");
    }

    #[test]
    fn test_read_no_newline_at_eof() {
        let data = b"a\tb";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);
        let mut records = Vec::new();

        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"a\tb");
    }

    #[test]
    fn test_read_crlf() {
        let data = b"a\tb\r\nc\td\r\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);
        let mut records = Vec::new();

        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], b"a\tb");
        assert_eq!(records[1], b"c\td");
    }

    #[test]
    fn test_reader_large_lines() {
        // Use a small initial buffer for test
        let data = "A".repeat(1000) + "\n" + &"B".repeat(2000) + "\n";
        let reader = std::io::Cursor::new(data.clone());
        let mut reader = TsvReader::with_capacity(reader, 10); // Too small for one record
        let mut records = Vec::new();

        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], "A".repeat(1000).as_bytes());
        assert_eq!(records[1], "B".repeat(2000).as_bytes());
    }

    #[test]
    fn test_for_each_row() {
        use crate::libs::tsv::record::Row;

        let data = b"A\tB\nC\tD\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);
        let mut rows = Vec::new();

        reader
            .for_each_row(b'\t', |row| {
                // Collect row content as strings for checking
                let mut row_data = Vec::new();
                // TsvRow doesn't expose len directly but we can guess or rely on get_str
                // Let's just grab known indices
                if let Some(s) = row.get_str(1) {
                    row_data.push(s.to_string());
                }
                if let Some(s) = row.get_str(2) {
                    row_data.push(s.to_string());
                }
                rows.push(row_data);
                Ok(())
            })
            .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["A", "B"]);
        assert_eq!(rows[1], vec!["C", "D"]);
    }

    #[test]
    fn test_read_header() {
        let data = b"h1\th2\nd1\td2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header = reader.read_header().unwrap().unwrap();
        assert_eq!(header, b"h1\th2");

        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"d1\td2");
    }

    #[test]
    fn test_read_header_empty() {
        let data = b"";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header = reader.read_header().unwrap();
        assert!(header.is_none());
    }

    #[test]
    fn test_copy_remainder() {
        let data = b"line1\nline2\nline3\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        // Read first line
        let header = reader.read_header().unwrap().unwrap();
        assert_eq!(header, b"line1");

        // Copy remainder
        let mut output = Vec::new();
        let count = reader.copy_remainder_to(&mut output).unwrap();

        assert_eq!(count, 12); // "line2\nline3\n" is 6+6=12 bytes
        assert_eq!(output, b"line2\nline3\n");
    }
}
