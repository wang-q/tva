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
        let plan = indices.and_then(|idxs| {
            if idxs.is_empty() {
                None
            } else {
                Some(SelectPlan::new(&idxs))
            }
        });

        Self {
            plan,
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
}
