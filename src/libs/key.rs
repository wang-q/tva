use crate::libs::select::SelectPlan;
use smallvec::SmallVec;
use std::ops::Range;

/// Buffer for storing key data, optimized for small keys.
pub type KeyBuffer = SmallVec<[u8; 32]>;

/// Represents a key extracted from a TSV line.
/// It can be a reference to a slice of the original line (zero-copy),
/// or an owned buffer (if the key is constructed from multiple fields or modified).
#[derive(Debug)]
pub enum ParsedKey<'a> {
    Ref(&'a [u8]),
    Owned(KeyBuffer),
}

impl<'a> AsRef<[u8]> for ParsedKey<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            ParsedKey::Ref(s) => s,
            ParsedKey::Owned(s) => s.as_ref(),
        }
    }
}

impl<'a> ParsedKey<'a> {
    /// Convert to an owned KeyBuffer.
    pub fn into_owned(self) -> KeyBuffer {
        match self {
            ParsedKey::Ref(s) => KeyBuffer::from_slice(s),
            ParsedKey::Owned(s) => s,
        }
    }
}

/// Helper struct to extract keys from TSV lines based on field selection.
pub struct KeyExtractor {
    plan: Option<SelectPlan>,
    // Store indices for efficient Row access (O(K) instead of O(N) scan)
    pub indices: Option<Vec<usize>>,
    ignore_case: bool,
    strict: bool,
    // Buffer for ranges to avoid allocation during extraction
    ranges_buf: Vec<Range<usize>>,
}

impl KeyExtractor {
    /// Create a new KeyExtractor.
    /// 
    /// * `indices`: List of field indices (0-based) to extract. If `None` or empty, the whole line is used as key.
    /// * `ignore_case`: If true, the key will be converted to lowercase.
    /// * `strict`: If true, `extract` returns error if any required field is missing. If false, missing fields are treated as empty.
    pub fn new(indices: Option<Vec<usize>>, ignore_case: bool, strict: bool) -> Self {
        let plan = indices.as_ref().and_then(|idxs| {
            if idxs.is_empty() {
                None
            } else {
                Some(SelectPlan::new(idxs))
            }
        });

        Self {
            plan,
            indices,
            ignore_case,
            strict,
            ranges_buf: Vec::with_capacity(8),
        }
    }

    /// Extract key from a line.
    /// Returns `Ok(ParsedKey)` on success.
    /// Returns `Err(missing_idx)` if a required field index is out of range and strict mode is enabled.
    pub fn extract<'a>(&mut self, line: &'a [u8], delimiter: u8) -> Result<ParsedKey<'a>, usize> {
        // Case 1: Whole line (no plan)
        if self.plan.is_none() {
            if self.ignore_case {
                let mut buf = KeyBuffer::with_capacity(line.len());
                // Simple ASCII lowercase mapping
                buf.extend(line.iter().map(|b| b.to_ascii_lowercase()));
                return Ok(ParsedKey::Owned(buf));
            } else {
                return Ok(ParsedKey::Ref(line));
            }
        }

        // Case 2: Selected fields
        let plan = self.plan.as_ref().unwrap();
        
        // Propagate error if field is missing and strict is true
        if let Err(idx) = plan.extract_ranges(line, delimiter, &mut self.ranges_buf) {
            if self.strict {
                return Err(idx);
            }
            // If not strict, ranges_buf contains empty ranges for missing fields (initialized by SelectPlan),
            // so we can proceed.
        }
        
        // Optimization: Single field and no ignore case -> Ref
        // But only if plan output length is 1.
        if plan.output_len() == 1 && !self.ignore_case {
            // ranges_buf should have size output_len
            if !self.ranges_buf.is_empty() {
                let range = &self.ranges_buf[0];
                if range.start >= range.end {
                    return Ok(ParsedKey::Ref(&[]));
                }
                // Safety check for bounds
                if range.end <= line.len() {
                    return Ok(ParsedKey::Ref(&line[range.clone()]));
                } else {
                    return Ok(ParsedKey::Ref(&[]));
                }
            }
        }

        // Construct key from ranges
        let mut key = KeyBuffer::new();
        let mut first = true;
        
        for range in self.ranges_buf.iter() {
            if !first {
                key.push(delimiter);
            }
            if range.start < range.end && range.end <= line.len() {
                let slice = &line[range.clone()];
                if self.ignore_case {
                    key.extend(slice.iter().map(|b| b.to_ascii_lowercase()));
                } else {
                    key.extend_from_slice(slice);
                }
            }
            first = false;
        }
        
        Ok(ParsedKey::Owned(key))
    }

    /// Extract key from a TsvRecord.
    /// This is optimized for records where fields are already parsed.
    /// Returns `Ok(ParsedKey)` on success.
    /// Returns `Err(missing_idx)` if a required field index is out of range and strict mode is enabled.
    pub fn extract_from_record<'a>(&mut self, record: &'a crate::libs::tsv::record::TsvRecord, delimiter: u8) -> Result<ParsedKey<'a>, usize> {
        // Case 1: Whole line
        if self.indices.is_none() {
            if self.ignore_case {
                let mut buf = KeyBuffer::with_capacity(record.as_line().len());
                buf.extend(record.as_line().iter().map(|b| b.to_ascii_lowercase()));
                return Ok(ParsedKey::Owned(buf));
            } else {
                return Ok(ParsedKey::Ref(record.as_line()));
            }
        }

        let indices = self.indices.as_ref().unwrap();

        // Optimization: Single field
        if indices.len() == 1 {
            let idx = indices[0]; // 0-based index
            let field = record.get(idx).unwrap_or(&[]);
            
            // Check strictness
            if self.strict && idx >= record.len() {
                return Err(idx);
            }

            if self.ignore_case {
                let mut buf = KeyBuffer::with_capacity(field.len());
                buf.extend(field.iter().map(|b| b.to_ascii_lowercase()));
                return Ok(ParsedKey::Owned(buf));
            } else {
                return Ok(ParsedKey::Ref(field));
            }
        }

        // Multiple fields
        let mut key = KeyBuffer::new();
        let mut first = true;

        for &idx in indices {
            if !first {
                key.push(delimiter);
            }
            
            let field = if idx < record.len() {
                record.get(idx).unwrap()
            } else {
                if self.strict {
                    return Err(idx);
                }
                &[]
            };

            if self.ignore_case {
                key.extend(field.iter().map(|b| b.to_ascii_lowercase()));
            } else {
                key.extend_from_slice(field);
            }
            first = false;
        }

        Ok(ParsedKey::Owned(key))
    }

    /// Extract key from a Row implementation.
    /// Note: `Row` trait uses 1-based indexing for `get_bytes`.
    /// `KeyExtractor` uses 0-based indices internally (from `SelectPlan` or `indices` arg).
    /// So we need to add 1 when calling `get_bytes`.
    pub fn extract_from_row<'a, R: crate::libs::tsv::record::Row + ?Sized>(&mut self, row: &'a R, delimiter: u8) -> Result<ParsedKey<'a>, usize> {
        // Case 1: Whole line
        if self.indices.is_none() {
            // Row trait doesn't expose whole line easily.
            // We assume caller handles whole line case or we fail.
            // But for `stats`, if no group-by, we don't call this.
            // If group-by is empty, `indices` is None? No, `new` handles empty -> None.
            // If `group-by` is not provided, `indices` is empty vec in `stats.rs`?
            // In `stats.rs`, `group_indices` is `Vec<usize>`.
            // If it's empty, `use_grouping` is false, and we don't extract key.
            // So for `stats`, we only call this when `indices` is Some/non-empty.
            
            // However, to be safe:
            return Ok(ParsedKey::Ref(&[])); // Or error?
        }

        let indices = self.indices.as_ref().unwrap();

        // Optimization: Single field
        if indices.len() == 1 {
            let idx = indices[0]; // 0-based index
            // Row uses 1-based index
            let field = row.get_bytes(idx + 1).unwrap_or(&[]);
            
            // Row::get_bytes returns None if out of bounds.
            // If field is None, it means index is out of bounds (or 0).
            // We check strictness.
            if self.strict && row.get_bytes(idx + 1).is_none() {
                 // Check if it's really out of bounds.
                 // We don't know row length easily from Row trait.
                 // But get_bytes returns None.
                 return Err(idx);
            }

            if self.ignore_case {
                let mut buf = KeyBuffer::with_capacity(field.len());
                buf.extend(field.iter().map(|b| b.to_ascii_lowercase()));
                return Ok(ParsedKey::Owned(buf));
            } else {
                return Ok(ParsedKey::Ref(field));
            }
        }

        // Multiple fields
        let mut key = KeyBuffer::new();
        let mut first = true;

        for &idx in indices {
            if !first {
                key.push(delimiter);
            }
            
            let field = row.get_bytes(idx + 1);
            let field_bytes = if let Some(f) = field {
                f
            } else {
                if self.strict {
                    return Err(idx);
                }
                &[] as &[u8]
            };

            if self.ignore_case {
                key.extend(field_bytes.iter().map(|b| b.to_ascii_lowercase()));
            } else {
                key.extend_from_slice(field_bytes);
            }
            first = false;
        }

        Ok(ParsedKey::Owned(key))
    }
}
