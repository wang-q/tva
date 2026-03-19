//! High-performance, zero-copy TSV reader.
//!
//! This module provides `TsvReader`, which manages an internal buffer to allow
//! iterating over TSV records with minimal allocation.

use crate::libs::tsv::header::{HeaderInfo, HeaderMode};
use crate::libs::tsv::record::TsvRow;
use std::io::{self, Read, Write};

#[cfg(target_arch = "x86_64")]
use crate::libs::tsv::sse2::Sse2Searcher;

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
    /// Cached separator positions for the current row (reused allocation).
    /// Using Option to delay initialization until first use.
    seps: Option<Vec<usize>>,
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
            seps: None, // Delay initialization until first use
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

    /// Reads header according to the specified mode.
    ///
    /// # Returns
    /// - `Ok(Some(HeaderInfo))` - Header was successfully detected
    /// - `Ok(None)` - No header found (empty file or no matching header)
    /// - `Err(e)` - I/O error occurred
    ///
    /// # Modes
    /// - `FirstLine`: First line is the header (even if empty)
    /// - `LinesN(n)`: First n lines are the header (including empty lines)
    /// - `HashLines`: Consecutive '#' lines (no column names)
    /// - `HashLines1`: Consecutive '#' lines + next line (column names)
    pub fn read_header_mode(
        &mut self,
        mode: HeaderMode,
    ) -> io::Result<Option<HeaderInfo>> {
        match mode {
            HeaderMode::FirstLine => self.read_header_first_line(),
            HeaderMode::LinesN(n) => self.read_header_lines_n(n),
            HeaderMode::HashLines => self.read_header_hash_lines(false),
            HeaderMode::HashLines1 => self.read_header_hash_lines(true),
        }
    }

    fn read_header_first_line(&mut self) -> io::Result<Option<HeaderInfo>> {
        let mut column_names = None;
        let res = self.for_each_record(|record| {
            // Take the first line as header (even if empty)
            column_names = Some(record.to_vec());
            Err(io::Error::new(io::ErrorKind::Interrupted, "Stop"))
        });

        match res {
            Ok(_) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                Ok(column_names.map(|line| HeaderInfo {
                    lines: Vec::new(),
                    column_names_line: Some(line),
                }))
            }
            Err(e) => Err(e),
        }
    }

    fn read_header_lines_n(&mut self, n: usize) -> io::Result<Option<HeaderInfo>> {
        let mut lines = Vec::new();
        let mut count = 0;

        let res = self.for_each_record(|record| {
            if count < n {
                lines.push(record.to_vec());
                count += 1;
                if count >= n {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Stop"));
                }
            }
            Ok(())
        });

        match res {
            Ok(_) => {
                if lines.is_empty() {
                    Ok(None)
                } else {
                    // LinesN mode: lines contains all N lines, column_names_line is None
                    Ok(Some(HeaderInfo {
                        lines,
                        column_names_line: None,
                    }))
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // LinesN mode: lines contains all N lines, column_names_line is None
                Ok(Some(HeaderInfo {
                    lines,
                    column_names_line: None,
                }))
            }
            Err(e) => Err(e),
        }
    }

    fn read_header_hash_lines(
        &mut self,
        include_next_line: bool,
    ) -> io::Result<Option<HeaderInfo>> {
        let mut lines = Vec::new();
        let mut column_names = None;
        let mut found_hash = false;
        let mut first_non_hash_line: Option<Vec<u8>> = None;

        let res = self.for_each_record(|record| {
            if record.starts_with(b"#") {
                lines.push(record.to_vec());
                found_hash = true;
            } else if include_next_line && found_hash && column_names.is_none() {
                // First non-hash line after hash lines is column names
                column_names = Some(record.to_vec());
                // Note: column names line is NOT included in lines (per documentation)
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Stop"));
            } else if !record.is_empty() {
                // Non-empty line that's not a hash line
                if !found_hash {
                    // No hash lines found
                    if include_next_line {
                        // For HashLines1 mode, use the first non-hash line as column names
                        column_names = Some(record.to_vec());
                        // Note: column names line is NOT included in lines (per documentation)
                        return Err(io::Error::new(io::ErrorKind::Interrupted, "Stop"));
                    } else {
                        // For HashLines mode, not a valid hash header
                        first_non_hash_line = Some(record.to_vec());
                        return Err(io::Error::other("No hash lines"));
                    }
                }
                // Hash lines ended - remember this line and stop
                // Don't use Interrupted here because for_each_record will skip the line
                first_non_hash_line = Some(record.to_vec());
                return Err(io::Error::other("Hash lines ended"));
            }
            // Empty lines are skipped
            Ok(())
        });

        // Note: When for_each_record returns an error (not Interrupted),
        // self.pos is NOT advanced past the current line. This means the next
        // for_each_record call will re-read the same line. This is the desired
        // behavior for HashLines mode - the first non-hash line should be
        // processed as data, not skipped.

        match res {
            Ok(_) => {
                if lines.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(HeaderInfo {
                        lines,
                        column_names_line: column_names,
                    }))
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // For HashLines1 mode without hash lines, lines is empty but column_names is set
                if lines.is_empty() && column_names.is_none() {
                    Ok(None)
                } else {
                    Ok(Some(HeaderInfo {
                        lines,
                        column_names_line: column_names,
                    }))
                }
            }
            Err(e) => {
                // For "Hash lines ended" error, we still return the header info
                if e.kind() == io::ErrorKind::Other && lines.is_empty() {
                    Ok(None)
                } else if e.kind() == io::ErrorKind::Other {
                    Ok(Some(HeaderInfo {
                        lines,
                        column_names_line: column_names,
                    }))
                } else {
                    Err(e)
                }
            }
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

    /// Read the next row with single-pass scanning.
    ///
    /// This method uses `memchr2` to simultaneously search for both tab and newline,
    /// eliminating the two-pass overhead of `for_each_record` + field splitting.
    ///
    /// Returns `Ok(Some(TsvRow))` if a row was read, `Ok(None)` at EOF.
    pub fn next_row(&mut self, delimiter: u8) -> io::Result<Option<TsvRow<'_, '_>>> {
        // Lazy initialization: create seps vector only when first used
        let seps = self.seps.get_or_insert_with(Vec::new);
        seps.clear();
        let mut line_start = self.pos;
        let mut field_start = self.pos;

        loop {
            let available = &self.buf[field_start..self.len];

            // Use memchr2 to find the next delimiter or newline
            match memchr::memchr2(delimiter, b'\n', available) {
                Some(offset) => {
                    let abs_pos = field_start + offset;
                    let byte = available[offset];

                    if byte == delimiter {
                        // Found delimiter - record field end position
                        seps.push(abs_pos - line_start);
                        field_start = abs_pos + 1;
                    } else {
                        // Found newline - complete the row
                        let line_end = abs_pos;

                        // Handle potential CR before LF
                        let content_end = if line_end > line_start
                            && line_end > 0
                            && self.buf[line_end - 1] == b'\r'
                        {
                            line_end - 1 - line_start
                        } else {
                            line_end - line_start
                        };

                        // Add final field end position
                        seps.push(content_end);
                        self.pos = abs_pos + 1;

                        // SAFETY: We return a TsvRow that references self.buf and self.seps.
                        // This is safe because seps is stored in self and lives as long as self.
                        // The caller must not hold the TsvRow across calls to next_row.
                        let row = TsvRow {
                            line: &self.buf[line_start..line_end],
                            ends: seps.as_slice(),
                        };
                        return Ok(Some(row));
                    }
                }
                None => {
                    // No delimiter or newline found in current buffer
                    if self.eof {
                        // Handle last record without newline
                        if field_start < self.len {
                            let line_end = self.len;

                            // Remove trailing CR if present
                            let content_end = if line_end > line_start
                                && line_end > 0
                                && self.buf[line_end - 1] == b'\r'
                            {
                                line_end.saturating_sub(1).saturating_sub(line_start)
                            } else {
                                line_end.saturating_sub(line_start)
                            };

                            seps.push(content_end);
                            self.pos = self.len;

                            let row = TsvRow {
                                line: &self.buf[line_start..line_end],
                                ends: seps.as_slice(),
                            };
                            return Ok(Some(row));
                        }
                        return Ok(None);
                    }

                    // Need to refill buffer
                    if field_start >= line_start {
                        // We have partial data - move it to front and continue
                        self.buf.copy_within(line_start..self.len, 0);
                        self.len -= line_start;
                        field_start -= line_start;
                    } else if field_start >= self.len {
                        field_start = 0;
                        self.len = 0;
                    }
                    self.pos = 0;
                    // After moving data, line_start is now at position 0
                    line_start = 0;

                    // Grow buffer if needed for large records
                    if self.len == self.buf.len() {
                        self.buf.resize(self.buf.len() * 2, 0);
                    }

                    // Read more data
                    let read_len = self.reader.read(&mut self.buf[self.len..])?;
                    if read_len == 0 {
                        self.eof = true;
                    } else {
                        self.len += read_len;
                    }
                }
            }
        }
    }

    /// Read the next row using SSE2-accelerated single-pass scanning (x86_64 only).
    ///
    /// This method uses hand-written SSE2 SIMD instructions to simultaneously search
    /// for tab, newline, and carriage return characters. Benchmarks show this achieves
    /// ~6.5 GiB/s throughput, which is ~670% faster than the standard two-pass approach.
    ///
    /// # Platform Support
    ///
    /// This method is only available on x86_64 platforms. On other platforms,
    /// use `next_row()` instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut reader = TsvReader::new(file);
    /// while let Some(row) = unsafe { reader.next_row_sse2() }? {
    ///     for field in row.fields() {
    ///         println!("{}", std::str::from_utf8(field).unwrap());
    ///     }
    /// }
    /// ```
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn next_row_sse2(&mut self) -> io::Result<Option<TsvRow<'_, '_>>> {
        let seps = self.seps.get_or_insert_with(Vec::new);
        seps.clear();

        let searcher = Sse2Searcher::new_tsv();
        let mut line_start = self.pos;

        loop {
            let available = &self.buf[self.pos..self.len];

            if available.is_empty() {
                if self.eof {
                    // Handle last record without newline
                    if line_start < self.len {
                        let line_end = self.len;
                        let content_end = if line_end > line_start
                            && line_end > 0
                            && self.buf[line_end - 1] == b'\r'
                        {
                            line_end - 1 - line_start
                        } else {
                            line_end - line_start
                        };
                        seps.push(content_end);
                        self.pos = self.len;

                        return Ok(Some(TsvRow {
                            line: &self.buf[line_start..line_end],
                            ends: seps.as_slice(),
                        }));
                    }
                    return Ok(None);
                }

                // Need to refill buffer
                if self.pos > 0 {
                    self.buf.copy_within(self.pos..self.len, 0);
                    self.len -= self.pos;
                    line_start = 0;
                    self.pos = 0;
                }

                // Grow buffer if needed
                if self.len == self.buf.len() {
                    self.buf.resize(self.buf.len() * 2, 0);
                }

                // Read more data
                let read_len = self.reader.read(&mut self.buf[self.len..])?;
                if read_len == 0 {
                    self.eof = true;
                } else {
                    self.len += read_len;
                }
                continue;
            }

            // Use SSE2 to find delimiters
            let mut found_newline = false;
            let mut newline_pos = 0;

            for pos in searcher.search(available) {
                let byte = available[pos];
                let abs_pos = self.pos + pos;

                if byte == b'\t' {
                    // Field delimiter
                    seps.push(abs_pos - line_start);
                } else if byte == b'\n' {
                    // Found newline - complete the row
                    found_newline = true;
                    newline_pos = abs_pos;
                    break;
                }
                // CR is handled with newline (we strip it at the end)
            }

            if found_newline {
                let line_end = newline_pos;

                // Handle potential CR before LF
                let content_end = if line_end > line_start
                    && line_end > 0
                    && self.buf[line_end - 1] == b'\r'
                {
                    line_end - 1 - line_start
                } else {
                    line_end - line_start
                };

                seps.push(content_end);
                self.pos = newline_pos + 1;

                return Ok(Some(TsvRow {
                    line: &self.buf[line_start..line_end],
                    ends: seps.as_slice(),
                }));
            }

            // No newline found - need more data
            // Move partial data to front and refill
            self.buf.copy_within(self.pos..self.len, 0);
            self.len -= self.pos;
            line_start = 0;
            self.pos = 0;

            // Grow buffer if needed
            if self.len == self.buf.len() {
                self.buf.resize(self.buf.len() * 2, 0);
            }

            // Read more data
            let read_len = self.reader.read(&mut self.buf[self.len..])?;
            if read_len == 0 {
                self.eof = true;
            } else {
                self.len += read_len;
            }
        }
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
    /// This method uses the optimized `next_row` internally for single-pass scanning.
    ///
    /// The delimiter parameter specifies the field separator (default is TAB).
    pub fn for_each_row<F>(&mut self, delimiter: u8, mut func: F) -> io::Result<()>
    where
        F: FnMut(&TsvRow) -> io::Result<()>,
    {
        while let Some(row) = self.next_row(delimiter)? {
            match func(&row) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    return Err(e);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
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
    fn test_for_each_record_error_propagation() {
        let data = b"a\tb\nc\td\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let result = reader.for_each_record(|_| {
            Err(io::Error::new(io::ErrorKind::Other, "test error"))
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_for_each_record_with_refill() {
        // Data larger than buffer to force refill
        let data = "a\tb\nc\td\ne\tf\n".repeat(1000);
        let cursor = Cursor::new(data.clone());
        let mut reader = TsvReader::with_capacity(cursor, 32); // Small buffer
        let mut records = Vec::new();

        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 3000);
    }

    #[test]
    fn test_for_each_record_with_refill2() {
        // Test that buffer refilling works correctly
        let data = b"line1\nline2\nline3\nline4\nline5\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::with_capacity(cursor, 16); // Small buffer

        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();

        assert_eq!(records.len(), 5);
        assert_eq!(records[0], b"line1");
        assert_eq!(records[4], b"line5");
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
    fn test_read_header_mode_first_line() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"col1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::FirstLine)
            .unwrap()
            .unwrap();
        assert!(header_info.lines.is_empty());
        assert_eq!(header_info.column_names_line, Some(b"col1\tcol2".to_vec()));

        // Verify data lines follow
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_first_line_with_empty() {
        use crate::libs::tsv::header::HeaderMode;

        // FirstLine mode now takes the first line even if empty
        let data = b"\n\ncol1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::FirstLine)
            .unwrap()
            .unwrap();
        // First line is empty
        assert_eq!(header_info.column_names_line, Some(b"".to_vec()));

        // Verify remaining data lines
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], b"");
        assert_eq!(records[1], b"col1\tcol2");
        assert_eq!(records[2], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_lines_n() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"comment1\ncomment2\ncol1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::LinesN(3))
            .unwrap()
            .unwrap();
        assert_eq!(header_info.lines.len(), 3);
        assert_eq!(header_info.lines[0], b"comment1");
        assert_eq!(header_info.lines[1], b"comment2");
        assert_eq!(header_info.lines[2], b"col1\tcol2");
        // LinesN mode: column_names_line is None
        assert_eq!(header_info.column_names_line, None);

        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_hash_lines() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"# Comment 1\n# Comment 2\ncol1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::HashLines)
            .unwrap()
            .unwrap();
        assert_eq!(header_info.lines.len(), 2);
        assert_eq!(header_info.lines[0], b"# Comment 1");
        assert_eq!(header_info.lines[1], b"# Comment 2");
        assert_eq!(header_info.column_names_line, None); // HashLines doesn't include column line

        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        // After fix: the first non-hash line is restored to buffer, so we see both lines
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], b"col1\tcol2");
        assert_eq!(records[1], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_hash_lines1() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"# Comment 1\n# Comment 2\ncol1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::HashLines1)
            .unwrap()
            .unwrap();
        assert_eq!(header_info.lines.len(), 2); // Only hash lines, column names NOT included
        assert_eq!(header_info.lines[0], b"# Comment 1");
        assert_eq!(header_info.lines[1], b"# Comment 2");
        assert_eq!(header_info.column_names_line, Some(b"col1\tcol2".to_vec()));

        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_empty_file() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::FirstLine).unwrap();
        assert!(header_info.is_none());
    }

    #[test]
    fn test_read_header_mode_no_hash_lines() {
        use crate::libs::tsv::header::HeaderMode;

        // File doesn't start with '#', so HashLines should return None
        let data = b"col1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::HashLines).unwrap();
        assert!(header_info.is_none());

        // Verify that data is still readable after HashLines returns None
        // The first line should NOT be consumed
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        // Both lines should be readable since no hash lines were found
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], b"col1\tcol2");
        assert_eq!(records[1], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_hash_lines_with_empty_lines() {
        use crate::libs::tsv::header::HeaderMode;

        // Hash lines with empty lines interspersed
        let data = b"# Comment 1\n\n# Comment 2\n\ncol1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::HashLines)
            .unwrap()
            .unwrap();
        // Empty lines are skipped, so we should have 2 hash lines
        assert_eq!(header_info.lines.len(), 2);
        assert_eq!(header_info.lines[0], b"# Comment 1");
        assert_eq!(header_info.lines[1], b"# Comment 2");
        assert_eq!(header_info.column_names_line, None);

        // Verify data lines follow
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        // Empty lines are skipped during header detection but may not be in data
        // We should see: col1\tcol2, val1\tval2 (empty lines may be skipped)
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], b"col1\tcol2");
        assert_eq!(records[1], b"val1\tval2");
    }

    #[test]
    fn test_read_header_mode_hash_lines_only_hash() {
        use crate::libs::tsv::header::HeaderMode;

        // File with only hash lines (no data)
        let data = b"# Comment 1\n# Comment 2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::HashLines)
            .unwrap()
            .unwrap();
        assert_eq!(header_info.lines.len(), 2);
        assert_eq!(header_info.lines[0], b"# Comment 1");
        assert_eq!(header_info.lines[1], b"# Comment 2");
        assert_eq!(header_info.column_names_line, None);

        // No data lines should follow
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_read_header_mode_hash_lines1_no_hash() {
        use crate::libs::tsv::header::HeaderMode;

        // File without hash lines for HashLines1 mode
        let data = b"col1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        // HashLines1 should use the first line as header when no hash lines found
        // lines should be empty (like FirstLine mode), column_names_line should be set
        let header_info = reader.read_header_mode(HeaderMode::HashLines1).unwrap();
        assert!(header_info.is_some());
        let info = header_info.unwrap();
        assert_eq!(info.lines.len(), 0); // Empty, like FirstLine mode
        assert_eq!(info.column_names_line, Some(b"col1\tcol2".to_vec()));

        // Remaining data should be readable
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"val1\tval2");
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

    // A mock reader that returns an error after a certain number of reads
    struct FailingReader {
        data: Vec<u8>,
        fail_after: usize,
        read_count: usize,
    }

    impl FailingReader {
        fn new(data: Vec<u8>, fail_after: usize) -> Self {
            Self {
                data,
                fail_after,
                read_count: 0,
            }
        }
    }

    impl std::io::Read for FailingReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read_count += 1;
            if self.read_count > self.fail_after {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated read error",
                ));
            }
            let len = std::cmp::min(buf.len(), self.data.len());
            buf[..len].copy_from_slice(&self.data[..len]);
            self.data.drain(..len);
            Ok(len)
        }
    }

    #[test]
    fn test_for_each_record_error_propagation_with_failing_reader() {
        // Test that non-Interrupted errors are properly propagated
        let data = b"header1\theader2\n".to_vec();
        let reader = FailingReader::new(data, 0);
        let mut tsv_reader = TsvReader::new(reader);

        let result: std::io::Result<()> = tsv_reader
            .for_each_record(|_rec| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Stop iteration",
                ))
            })
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    Ok(())
                } else {
                    Err(e)
                }
            });

        // Should fail with the simulated error, not Interrupted
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
        assert!(err.to_string().contains("Simulated read error"));
    }

    #[test]
    fn test_read_header_empty_file() {
        let data = b"";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header = reader.read_header().unwrap();
        assert!(header.is_none());
    }

    #[test]
    fn test_read_header_first_line_empty_lines_only() {
        // File with only empty lines - read_header returns first empty line
        // because it doesn't skip empty lines (unlike read_header_mode with FirstLine)
        let data = b"\n\n\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header = reader.read_header().unwrap();
        // read_header returns the first line (even if empty)
        assert!(header.is_some());
        assert!(header.unwrap().is_empty());
    }

    #[test]
    fn test_read_header_lines_n_empty_file() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::LinesN(3)).unwrap();
        assert!(header_info.is_none());
    }

    #[test]
    fn test_read_header_lines_n_insufficient_lines() {
        use crate::libs::tsv::header::HeaderMode;

        // Only 2 lines but requesting 3
        let data = b"line1\nline2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::LinesN(3)).unwrap();
        // Should return what we have
        assert!(header_info.is_some());
        assert_eq!(header_info.unwrap().lines.len(), 2);
    }

    #[test]
    fn test_read_header_hash_lines_only_empty() {
        use crate::libs::tsv::header::HeaderMode;

        // File with only empty lines before hash lines
        let data = b"\n\n# Comment\ndata\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::HashLines).unwrap();
        assert!(header_info.is_some());
        assert_eq!(header_info.unwrap().lines.len(), 1);
    }

    #[test]
    fn test_read_header_hash_lines1_only_hash() {
        use crate::libs::tsv::header::HeaderMode;

        // Only hash lines, no column names
        let data = b"# Comment 1\n# Comment 2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader.read_header_mode(HeaderMode::HashLines1).unwrap();
        assert!(header_info.is_some());
        // Should have 2 hash lines but no column_names_line
        let info = header_info.unwrap();
        assert_eq!(info.lines.len(), 2);
        assert!(info.column_names_line.is_none());
    }

    #[test]
    fn test_copy_remainder_empty() {
        let data = b"line1\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        // Read everything
        let _ = reader.read_header().unwrap();

        // Copy remainder (should be empty)
        let mut output = Vec::new();
        let count = reader.copy_remainder_to(&mut output).unwrap();

        assert_eq!(count, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn test_read_header_mode_lines_n_single_line() {
        use crate::libs::tsv::header::HeaderMode;

        let data = b"col1\tcol2\nval1\tval2\n";
        let cursor = Cursor::new(data);
        let mut reader = TsvReader::new(cursor);

        let header_info = reader
            .read_header_mode(HeaderMode::LinesN(1))
            .unwrap()
            .unwrap();
        assert_eq!(header_info.lines.len(), 1);
        // LinesN mode: column_names_line is None
        assert_eq!(header_info.column_names_line, None);

        // Verify data follows
        let mut records = Vec::new();
        reader
            .for_each_record(|rec| {
                records.push(rec.to_vec());
                Ok(())
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], b"val1\tval2");
    }
}
