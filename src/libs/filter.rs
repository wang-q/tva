//! Field-based filter engine shared by `tva filter` and other commands.
//!
//! This module turns `<field-list>:value` specifications into test operators
//! and evaluates them on a single row of fields.
//!
//! Basic example: split a `<field-list>:value` specification:
//!
//! ```
//! use tva::libs::filter::split_spec;
//!
//! let (fields, value) = split_spec("2-3:10").unwrap();
//! assert_eq!(fields, "2-3");
//! assert_eq!(value, "10");
//! ```
//!
//! Numeric filtering on a single row:
//!
//! ```
//! use tva::libs::filter::{TestKind, NumericOp};
//! use tva::libs::tsv::record::StrSliceRow;
//!
//! let row = StrSliceRow { fields: &["id", "10.5"] };
//! let test = TestKind::NumericCmp {
//!     fields: vec![2],
//!     op: NumericOp::Gt,
//!     value: 10.0,
//! };
//!
//! assert!(test.eval_row(&row));
//! ```
//!
//! Substring matching on a single row:
//!
//! ```
//! use tva::libs::filter::TestKind;
//! use tva::libs::tsv::record::StrSliceRow;
//!
//! let row = StrSliceRow { fields: &["foo", "barbaz"] };
//! let test = TestKind::StrIn {
//!     fields: vec![2],
//!     value: "bar".to_string(),
//!     case_insensitive: false,
//!     negated: false,
//! };
//!
//! assert!(test.eval_row(&row));
//! ```

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use std::io::Write;
use anyhow::Result;
use memchr::memchr_iter;
use crate::libs::io::map_io_err;
use crate::libs::tsv::record::{Row, TsvRow, StrSliceRow};

#[derive(Default)]
pub struct FilterConfig {
    pub delimiter: char,
    pub has_header: bool,
    pub use_or: bool,
    pub invert: bool,
    pub count_only: bool,
    pub line_buffered: bool,
    pub label_header: Option<String>,
    pub label_pass_val: String,
    pub label_fail_val: String,

    pub empty_specs: Vec<String>,
    pub not_empty_specs: Vec<String>,
    pub blank_specs: Vec<String>,
    pub not_blank_specs: Vec<String>,
    pub numeric_specs: Vec<PendingNumeric>,
    pub str_cmp_specs: Vec<PendingStrCmp>,
    pub char_len_specs: Vec<PendingCharLen>,
    pub byte_len_specs: Vec<PendingByteLen>,
    pub numeric_prop_specs: Vec<PendingNumericProp>,
    pub str_eq_specs: Vec<PendingStrEq>,
    pub substr_specs: Vec<PendingSubstr>,
    pub regex_specs: Vec<PendingRegex>,
    pub ff_numeric_specs: Vec<PendingFieldFieldNumeric>,
    pub ff_str_specs: Vec<PendingFieldFieldStr>,
    pub ff_absdiff_specs: Vec<PendingFieldFieldAbsDiff>,
    pub ff_reldiff_specs: Vec<PendingFieldFieldRelDiff>,
}

impl FilterConfig {
    pub fn as_spec_config(&self) -> FilterSpecConfig<'_> {
        FilterSpecConfig {
            empty_specs: &self.empty_specs,
            not_empty_specs: &self.not_empty_specs,
            blank_specs: &self.blank_specs,
            not_blank_specs: &self.not_blank_specs,
            numeric_specs: &self.numeric_specs,
            str_cmp_specs: &self.str_cmp_specs,
            char_len_specs: &self.char_len_specs,
            byte_len_specs: &self.byte_len_specs,
            numeric_prop_specs: &self.numeric_prop_specs,
            str_eq_specs: &self.str_eq_specs,
            substr_specs: &self.substr_specs,
            regex_specs: &self.regex_specs,
            ff_numeric_specs: &self.ff_numeric_specs,
            ff_str_specs: &self.ff_str_specs,
            ff_absdiff_specs: &self.ff_absdiff_specs,
            ff_reldiff_specs: &self.ff_reldiff_specs,
        }
    }
}

pub fn run_filter<W: Write>(
    infiles: &[String],
    writer: &mut W,
    config: FilterConfig,
) -> Result<()> {
    let mut total_matched: u64 = 0;
    let mut header_written = false;
    let mut delim_buf = [0u8; 4];
    let delim_bytes = config.delimiter.encode_utf8(&mut delim_buf).as_bytes();
    let delim_byte = config.delimiter as u8;

    let tests_without_header: Option<Vec<TestKind>> = if config.has_header {
        None
    } else {
        Some(
            build_tests(None, config.delimiter, config.as_spec_config()).map_err(|e| anyhow::anyhow!(e))?,
        )
    };

    let max_field_without_header: usize = tests_without_header
        .as_ref()
        .map(|tests| tests.iter().map(|t| t.max_field_index()).max().unwrap_or(0))
        .unwrap_or(0);

    for input in crate::libs::io::input_sources(infiles) {
        let mut tsv_reader = crate::libs::tsv::reader::TsvReader::new(input.reader);
        let mut tests_with_header: Option<Vec<TestKind>> = None;
        let mut max_field_for_rows = max_field_without_header;
        let mut ends: Vec<usize> = Vec::new();

        if config.has_header {
            if let Some(header_bytes) = tsv_reader.read_header().map_err(map_io_err)? {
                let header_line =
                    std::str::from_utf8(&header_bytes).map_err(map_io_err)?;
                let header =
                    crate::libs::tsv::fields::Header::from_line(header_line, config.delimiter);

                if !header_written && !config.count_only {
                    if let Some(ref lbl) = config.label_header {
                        writer.write_all(&header_bytes)?;
                        writer.write_all(delim_bytes)?;
                        writer.write_all(lbl.as_bytes())?;
                        writer.write_all(b"\n")?;
                    } else {
                        writer.write_all(&header_bytes)?;
                        writer.write_all(b"\n")?;
                    }
                    if config.line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }

                let tests = build_tests(Some(&header), config.delimiter, config.as_spec_config())
                    .map_err(|e| anyhow::anyhow!(e))?;
                max_field_for_rows =
                    tests.iter().map(|t| t.max_field_index()).max().unwrap_or(0);
                tests_with_header = Some(tests);
            }
        }

        tsv_reader.for_each_record(|record| {
            let tests: &[TestKind] = if config.has_header {
                match tests_with_header.as_ref() {
                    Some(v) => v.as_slice(),
                    None => return Ok(()),
                }
            } else {
                tests_without_header.as_ref().unwrap().as_slice()
            };

            let mut row_match = if tests.is_empty() {
                true
            } else {
                ends.clear();
                if max_field_for_rows > 0 {
                    let mut count = 0usize;
                    for pos in memchr_iter(delim_byte, record) {
                        ends.push(pos);
                        count += 1;
                        if count >= max_field_for_rows {
                            break;
                        }
                    }
                }

                let row = TsvRow {
                    line: record,
                    ends: &ends,
                };

                if config.use_or {
                    let mut any = false;
                    for t in tests {
                        if t.eval_row(&row) {
                            any = true;
                            break;
                        }
                    }
                    any
                } else {
                    let mut all = true;
                    for t in tests {
                        if !t.eval_row(&row) {
                            all = false;
                            break;
                        }
                    }
                    all
                }
            };

            if config.invert {
                row_match = !row_match;
            }

            if config.label_header.is_some() {
                let val = if row_match {
                    &config.label_pass_val
                } else {
                    &config.label_fail_val
                };
                writer.write_all(record)?;
                writer.write_all(delim_bytes)?;
                writer.write_all(val.as_bytes())?;
                writer.write_all(b"\n")?;
                if config.line_buffered {
                    writer.flush()?;
                }
            } else if row_match {
                if config.count_only {
                    total_matched += 1;
                } else {
                    writer.write_all(record)?;
                    writer.write_all(b"\n")?;
                    if config.line_buffered {
                        writer.flush()?;
                    }
                }
            }

            Ok(())
        })?;
    }

    if config.count_only {
        // println! is not generic over writer, so we use writeln! or writer.write
        // But the original code used println!.
        // Here we should write to the writer if possible, but println! writes to stdout.
        // Since writer can be a File or Stdout, we should use writeln!(writer, ...).
        writeln!(writer, "{}", total_matched)?;
    }

    Ok(())
}

pub enum NumericOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

pub enum NumericProp {
    IsNumeric,
    IsFinite,
    IsNaN,
    IsInfinity,
}

pub enum TestKind {
    Empty {
        fields: Vec<usize>,
    },
    NotEmpty {
        fields: Vec<usize>,
    },
    Blank {
        fields: Vec<usize>,
    },
    NotBlank {
        fields: Vec<usize>,
    },
    NumericCmp {
        fields: Vec<usize>,
        op: NumericOp,
        value: f64,
    },
    CharLenCmp {
        fields: Vec<usize>,
        op: NumericOp,
        value: f64,
    },
    ByteLenCmp {
        fields: Vec<usize>,
        op: NumericOp,
        value: f64,
    },
    NumericPropTest {
        fields: Vec<usize>,
        prop: NumericProp,
    },
    StrEq {
        fields: Vec<usize>,
        value: String,
        case_insensitive: bool,
    },
    StrNe {
        fields: Vec<usize>,
        value: String,
        case_insensitive: bool,
    },
    StrCmp {
        fields: Vec<usize>,
        op: NumericOp,
        value: String,
    },
    StrIn {
        fields: Vec<usize>,
        value: String,
        case_insensitive: bool,
        negated: bool,
    },
    Regex {
        fields: Vec<usize>,
        regex: Regex,
        negated: bool,
    },
    FieldFieldNumericCmp {
        left_fields: Vec<usize>,
        right_fields: Vec<usize>,
        op: NumericOp,
    },
    FieldFieldStrCmp {
        left_fields: Vec<usize>,
        right_fields: Vec<usize>,
        case_insensitive: bool,
        negated: bool,
    },
    FieldFieldAbsDiffCmp {
        left_fields: Vec<usize>,
        right_fields: Vec<usize>,
        op: NumericOp, // Only Le or Gt are expected
        value: f64,
    },
    FieldFieldRelDiffCmp {
        left_fields: Vec<usize>,
        right_fields: Vec<usize>,
        op: NumericOp, // Only Le or Gt are expected
        value: f64,
    },
}

impl TestKind {
    pub fn eval(&self, fields: &[&str]) -> bool {
        self.eval_row(&StrSliceRow { fields })
    }

    pub fn eval_row<R: Row + ?Sized>(&self, row: &R) -> bool {
        match self {
            TestKind::Empty { fields: idxs } => idxs
                .iter()
                .all(|idx| row.get_bytes(*idx).map(|b| b.is_empty()).unwrap_or(true)),
            TestKind::NotEmpty { fields: idxs } => idxs
                .iter()
                .all(|idx| row.get_bytes(*idx).map(|b| !b.is_empty()).unwrap_or(false)),
            TestKind::Blank { fields: idxs } => idxs.iter().all(|idx| {
                row.get_str(*idx)
                    .map(|s| s.chars().all(|c| c.is_whitespace()))
                    .unwrap_or(true)
            }),
            TestKind::NotBlank { fields: idxs } => idxs.iter().all(|idx| {
                row.get_str(*idx)
                    .map(|s| s.chars().any(|c| !c.is_whitespace()))
                    .unwrap_or(false)
            }),
            TestKind::NumericCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let s = match row.get_str(*idx) {
                    Some(v) => v,
                    None => return false,
                };
                let parsed = match s.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                match op {
                    NumericOp::Gt => parsed > *value,
                    NumericOp::Ge => parsed >= *value,
                    NumericOp::Lt => parsed < *value,
                    NumericOp::Le => parsed <= *value,
                    NumericOp::Eq => parsed == *value,
                    NumericOp::Ne => parsed != *value,
                }
            }),
            TestKind::CharLenCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let s = row.get_str(*idx).unwrap_or("");
                let len = s.graphemes(true).count() as f64;
                match op {
                    NumericOp::Gt => len > *value,
                    NumericOp::Ge => len >= *value,
                    NumericOp::Lt => len < *value,
                    NumericOp::Le => len <= *value,
                    NumericOp::Eq => len == *value,
                    NumericOp::Ne => len != *value,
                }
            }),
            TestKind::ByteLenCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let len = row.get_bytes(*idx).unwrap_or(&[]).len() as f64;
                match op {
                    NumericOp::Gt => len > *value,
                    NumericOp::Ge => len >= *value,
                    NumericOp::Lt => len < *value,
                    NumericOp::Le => len <= *value,
                    NumericOp::Eq => len == *value,
                    NumericOp::Ne => len != *value,
                }
            }),
            TestKind::NumericPropTest { fields: idxs, prop } => idxs.iter().all(|idx| {
                let s = match row.get_str(*idx) {
                    Some(v) => v,
                    None => return false,
                };
                let parsed = match s.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                match prop {
                    NumericProp::IsNumeric => true,
                    NumericProp::IsFinite => parsed.is_finite(),
                    NumericProp::IsNaN => parsed.is_nan(),
                    NumericProp::IsInfinity => parsed.is_infinite(),
                }
            }),
            TestKind::StrEq {
                fields: idxs,
                value,
                case_insensitive,
            } => idxs.iter().all(|idx| match row.get_str(*idx) {
                Some(s) => {
                    if *case_insensitive {
                        s.to_lowercase() == value.to_lowercase()
                    } else {
                        s == value
                    }
                }
                None => false,
            }),
            TestKind::StrNe {
                fields: idxs,
                value,
                case_insensitive,
            } => idxs.iter().all(|idx| match row.get_str(*idx) {
                Some(s) => {
                    if *case_insensitive {
                        s.to_lowercase() != value.to_lowercase()
                    } else {
                        s != value
                    }
                }
                None => true,
            }),
            TestKind::StrCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| match row.get_str(*idx) {
                Some(s) => match op {
                    NumericOp::Gt => s > value.as_str(),
                    NumericOp::Ge => s >= value.as_str(),
                    NumericOp::Lt => s < value.as_str(),
                    NumericOp::Le => s <= value.as_str(),
                    NumericOp::Eq => s == value.as_str(),
                    NumericOp::Ne => s != value.as_str(),
                },
                None => false,
            }),
            TestKind::StrIn {
                fields: idxs,
                value,
                case_insensitive,
                negated,
            } => idxs.iter().all(|idx| {
                let haystack = row.get_str(*idx).unwrap_or("");
                let found = if *case_insensitive {
                    haystack.to_lowercase().contains(&value.to_lowercase())
                } else {
                    haystack.contains(value)
                };
                if *negated {
                    !found
                } else {
                    found
                }
            }),
            TestKind::Regex {
                fields: idxs,
                regex,
                negated,
            } => idxs.iter().all(|idx| {
                let s = row.get_str(*idx).unwrap_or("");
                let matched = regex.is_match(s);
                if *negated {
                    !matched
                } else {
                    matched
                }
            }),
            TestKind::FieldFieldNumericCmp {
                left_fields,
                right_fields,
                op,
            } => {
                if left_fields.len() != right_fields.len() {
                    return false;
                }
                left_fields.iter().zip(right_fields.iter()).all(|(l, r)| {
                    let l_s = match row.get_str(*l) {
                        Some(v) => v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_s = match row.get_str(*r) {
                            Some(v) => v,
                            None => return false,
                        };
                        match r_s.parse::<f64>() {
                            Ok(v) => v,
                            Err(_) => return false,
                        }
                    };
                    match op {
                        NumericOp::Gt => l_v > r_v,
                        NumericOp::Ge => l_v >= r_v,
                        NumericOp::Lt => l_v < r_v,
                        NumericOp::Le => l_v <= r_v,
                        NumericOp::Eq => l_v == r_v,
                        NumericOp::Ne => l_v != r_v,
                    }
                })
            }
            TestKind::FieldFieldStrCmp {
                left_fields,
                right_fields,
                case_insensitive,
                negated,
            } => {
                if left_fields.len() != right_fields.len() {
                    return false;
                }
                left_fields.iter().zip(right_fields.iter()).all(|(l, r)| {
                    if l == r {
                        return !negated;
                    }
                    let l_s = row.get_str(*l).unwrap_or("");
                    let r_s = row.get_str(*r).unwrap_or("");
                    let eq = if *case_insensitive {
                        l_s.to_lowercase() == r_s.to_lowercase()
                    } else {
                        l_s == r_s
                    };
                    if *negated {
                        !eq
                    } else {
                        eq
                    }
                })
            }
            TestKind::FieldFieldAbsDiffCmp {
                left_fields,
                right_fields,
                op,
                value,
            } => {
                if left_fields.len() != right_fields.len() {
                    return false;
                }
                left_fields.iter().zip(right_fields.iter()).all(|(l, r)| {
                    let l_s = match row.get_str(*l) {
                        Some(v) => v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_s = match row.get_str(*r) {
                            Some(v) => v,
                            None => return false,
                        };
                        match r_s.parse::<f64>() {
                            Ok(v) => v,
                            Err(_) => return false,
                        }
                    };
                    let diff = (l_v - r_v).abs();
                    match op {
                        NumericOp::Le => diff <= *value,
                        NumericOp::Gt => diff > *value,
                        NumericOp::Ge
                        | NumericOp::Lt
                        | NumericOp::Eq
                        | NumericOp::Ne => false,
                    }
                })
            }
            TestKind::FieldFieldRelDiffCmp {
                left_fields,
                right_fields,
                op,
                value,
            } => {
                if left_fields.len() != right_fields.len() {
                    return false;
                }
                left_fields.iter().zip(right_fields.iter()).all(|(l, r)| {
                    let l_s = match row.get_str(*l) {
                        Some(v) => v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_s = match row.get_str(*r) {
                            Some(v) => v,
                            None => return false,
                        };
                        match r_s.parse::<f64>() {
                            Ok(v) => v,
                            Err(_) => return false,
                        }
                    };

                    if l_v == r_v {
                        match op {
                            NumericOp::Le => 0.0 <= *value,
                            NumericOp::Gt => 0.0 > *value,
                            _ => false,
                        }
                    } else {
                        let denom = l_v.abs().min(r_v.abs());
                        let rel = if denom == 0.0 {
                            f64::INFINITY
                        } else {
                            (l_v - r_v).abs() / denom
                        };
                        match op {
                            NumericOp::Le => rel <= *value,
                            NumericOp::Gt => rel > *value,
                            NumericOp::Ge
                            | NumericOp::Lt
                            | NumericOp::Eq
                            | NumericOp::Ne => false,
                        }
                    }
                })
            }
        }
    }

    pub fn max_field_index(&self) -> usize {
        match self {
            TestKind::Empty { fields }
            | TestKind::NotEmpty { fields }
            | TestKind::Blank { fields }
            | TestKind::NotBlank { fields }
            | TestKind::NumericCmp { fields, .. }
            | TestKind::CharLenCmp { fields, .. }
            | TestKind::ByteLenCmp { fields, .. }
            | TestKind::NumericPropTest { fields, .. }
            | TestKind::StrEq { fields, .. }
            | TestKind::StrNe { fields, .. }
            | TestKind::StrCmp { fields, .. }
            | TestKind::StrIn { fields, .. }
            | TestKind::Regex { fields, .. } => {
                fields.iter().copied().max().unwrap_or(0)
            }
            TestKind::FieldFieldNumericCmp {
                left_fields,
                right_fields,
                ..
            }
            | TestKind::FieldFieldStrCmp {
                left_fields,
                right_fields,
                ..
            }
            | TestKind::FieldFieldAbsDiffCmp {
                left_fields,
                right_fields,
                ..
            }
            | TestKind::FieldFieldRelDiffCmp {
                left_fields,
                right_fields,
                ..
            } => left_fields
                .iter()
                .chain(right_fields.iter())
                .copied()
                .max()
                .unwrap_or(0),
        }
    }
}

pub struct PendingNumeric {
    pub spec: String,
    pub op: NumericOp,
}

pub struct PendingStrEq {
    pub spec: String,
    pub case_insensitive: bool,
    pub negated: bool,
}

pub struct PendingStrCmp {
    pub spec: String,
    pub op: NumericOp,
}

pub struct PendingSubstr {
    pub spec: String,
    pub case_insensitive: bool,
    pub negated: bool,
}

pub struct PendingRegex {
    pub spec: String,
    pub case_insensitive: bool,
    pub negated: bool,
}

pub struct PendingCharLen {
    pub spec: String,
    pub op: NumericOp,
}

pub struct PendingByteLen {
    pub spec: String,
    pub op: NumericOp,
}

pub struct PendingNumericProp {
    pub spec: String,
    pub prop: NumericProp,
}

pub struct PendingFieldFieldNumeric {
    pub spec: String,
    pub op: NumericOp,
}

pub struct PendingFieldFieldStr {
    pub spec: String,
    pub case_insensitive: bool,
    pub negated: bool,
}

pub struct PendingFieldFieldAbsDiff {
    pub spec: String,  // FIELD1:FIELD2:NUM
    pub op: NumericOp, // Le or Gt
}

pub struct PendingFieldFieldRelDiff {
    pub spec: String,  // FIELD1:FIELD2:NUM
    pub op: NumericOp, // Le or Gt
}

pub fn split_spec(spec: &str) -> Result<(String, String), String> {
    if let Some(pos) = spec.rfind(':') {
        let (left, right) = spec.split_at(pos);
        Ok((left.to_string(), right[1..].to_string()))
    } else if let Some(pos) = spec.rfind(' ') {
        let (left, right) = spec.split_at(pos);
        Ok((left.to_string(), right[1..].to_string()))
    } else {
        Err(format!(
            "missing `:` separator in `{}` (expected <field-list>:<value>)",
            spec
        ))
    }
}

pub struct FilterSpecConfig<'a> {
    pub empty_specs: &'a [String],
    pub not_empty_specs: &'a [String],
    pub blank_specs: &'a [String],
    pub not_blank_specs: &'a [String],
    pub numeric_specs: &'a [PendingNumeric],
    pub str_cmp_specs: &'a [PendingStrCmp],
    pub char_len_specs: &'a [PendingCharLen],
    pub byte_len_specs: &'a [PendingByteLen],
    pub numeric_prop_specs: &'a [PendingNumericProp],
    pub str_eq_specs: &'a [PendingStrEq],
    pub substr_specs: &'a [PendingSubstr],
    pub regex_specs: &'a [PendingRegex],
    pub ff_numeric_specs: &'a [PendingFieldFieldNumeric],
    pub ff_str_specs: &'a [PendingFieldFieldStr],
    pub ff_absdiff_specs: &'a [PendingFieldFieldAbsDiff],
    pub ff_reldiff_specs: &'a [PendingFieldFieldRelDiff],
}

pub fn build_tests(
    header: Option<&crate::libs::tsv::fields::Header>,
    delimiter: char,
    config: FilterSpecConfig,
) -> Result<Vec<TestKind>, String> {
    let mut tests = Vec::new();

    for spec in config.empty_specs {
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            spec, header, delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::Empty { fields: vec![idx] });
        }
    }

    for spec in config.not_empty_specs {
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            spec, header, delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::NotEmpty { fields: vec![idx] });
        }
    }

    for spec in config.blank_specs {
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            spec, header, delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::Blank { fields: vec![idx] });
        }
    }

    for spec in config.not_blank_specs {
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            spec, header, delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::NotBlank { fields: vec![idx] });
        }
    }

    for p in config.numeric_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        if idxs.is_empty() {
            return Err(format!("field list cannot be empty in `{}`", p.spec));
        }
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid numeric value `{}` in `{}`", value_part, p.spec)
        })?;
        for idx in idxs {
            tests.push(TestKind::NumericCmp {
                fields: vec![idx],
                op: match p.op {
                    NumericOp::Gt => NumericOp::Gt,
                    NumericOp::Ge => NumericOp::Ge,
                    NumericOp::Lt => NumericOp::Lt,
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Eq => NumericOp::Eq,
                    NumericOp::Ne => NumericOp::Ne,
                },
                value,
            });
        }
    }

    for p in config.str_cmp_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::StrCmp {
                fields: vec![idx],
                op: match p.op {
                    NumericOp::Gt => NumericOp::Gt,
                    NumericOp::Ge => NumericOp::Ge,
                    NumericOp::Lt => NumericOp::Lt,
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Eq => NumericOp::Eq,
                    NumericOp::Ne => NumericOp::Ne,
                },
                value: value_part.to_string(),
            });
        }
    }

    for p in config.char_len_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid length value `{}` in `{}`", value_part, p.spec)
        })?;
        for idx in idxs {
            tests.push(TestKind::CharLenCmp {
                fields: vec![idx],
                op: match p.op {
                    NumericOp::Gt => NumericOp::Gt,
                    NumericOp::Ge => NumericOp::Ge,
                    NumericOp::Lt => NumericOp::Lt,
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Eq => NumericOp::Eq,
                    NumericOp::Ne => NumericOp::Ne,
                },
                value,
            });
        }
    }

    for p in config.byte_len_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid length value `{}` in `{}`", value_part, p.spec)
        })?;
        for idx in idxs {
            tests.push(TestKind::ByteLenCmp {
                fields: vec![idx],
                op: match p.op {
                    NumericOp::Gt => NumericOp::Gt,
                    NumericOp::Ge => NumericOp::Ge,
                    NumericOp::Lt => NumericOp::Lt,
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Eq => NumericOp::Eq,
                    NumericOp::Ne => NumericOp::Ne,
                },
                value,
            });
        }
    }

    for p in config.numeric_prop_specs {
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &p.spec, header, delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::NumericPropTest {
                fields: vec![idx],
                prop: match p.prop {
                    NumericProp::IsNumeric => NumericProp::IsNumeric,
                    NumericProp::IsFinite => NumericProp::IsFinite,
                    NumericProp::IsNaN => NumericProp::IsNaN,
                    NumericProp::IsInfinity => NumericProp::IsInfinity,
                },
            });
        }
    }

    for p in config.str_eq_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        if p.negated {
            for idx in idxs {
                tests.push(TestKind::StrNe {
                    fields: vec![idx],
                    value: value_part.clone(),
                    case_insensitive: p.case_insensitive,
                });
            }
        } else {
            for idx in idxs {
                tests.push(TestKind::StrEq {
                    fields: vec![idx],
                    value: value_part.clone(),
                    case_insensitive: p.case_insensitive,
                });
            }
        }
    }

    for p in config.substr_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        for idx in idxs {
            tests.push(TestKind::StrIn {
                fields: vec![idx],
                value: value_part.clone(),
                case_insensitive: p.case_insensitive,
                negated: p.negated,
            });
        }
    }

    for p in config.regex_specs {
        let (field_part, pattern) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        let regex = if p.case_insensitive {
            Regex::new(&format!("(?i:{})", pattern))
        } else {
            Regex::new(&pattern)
        }
        .map_err(|e| format!("invalid regex `{}`: {}", pattern, e))?;
        for idx in idxs {
            tests.push(TestKind::Regex {
                fields: vec![idx],
                regex: regex.clone(),
                negated: p.negated,
            });
        }
    }

    for p in config.ff_numeric_specs {
        let (left_part, right_part) = split_spec(&p.spec)?;
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left_part, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right_part,
            header,
            delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list lengths in `{}` (left has {}, right has {})",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        for (l, r) in left_idxs.iter().zip(right_idxs.iter()) {
            tests.push(TestKind::FieldFieldNumericCmp {
                left_fields: vec![*l],
                right_fields: vec![*r],
                op: match p.op {
                    NumericOp::Gt => NumericOp::Gt,
                    NumericOp::Ge => NumericOp::Ge,
                    NumericOp::Lt => NumericOp::Lt,
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Eq => NumericOp::Eq,
                    NumericOp::Ne => NumericOp::Ne,
                },
            });
        }
    }

    for p in config.ff_str_specs {
        let (left_part, right_part) = split_spec(&p.spec)?;
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left_part, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right_part,
            header,
            delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list lengths in `{}` (left has {}, right has {})",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        for (l, r) in left_idxs.iter().zip(right_idxs.iter()) {
            tests.push(TestKind::FieldFieldStrCmp {
                left_fields: vec![*l],
                right_fields: vec![*r],
                case_insensitive: p.case_insensitive,
                negated: p.negated,
            });
        }
    }

    for p in config.ff_absdiff_specs {
        let (left_and_right, value_part) = split_spec(&p.spec)?;
        let (left_part, right_part) = if let Some(pos) = left_and_right.rfind(':') {
            (&left_and_right[..pos], &left_and_right[pos + 1..])
        } else {
            return Err(format!(
                "missing second `:` in `{}` (expected FIELD1:FIELD2:NUM)",
                p.spec
            ));
        };
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            left_part, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            right_part, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list lengths in `{}` (left has {}, right has {})",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid numeric value `{}` in `{}`", value_part, p.spec)
        })?;
        for (l, r) in left_idxs.iter().zip(right_idxs.iter()) {
            tests.push(TestKind::FieldFieldAbsDiffCmp {
                left_fields: vec![*l],
                right_fields: vec![*r],
                op: match p.op {
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Gt => NumericOp::Gt,
                    _ => return Err("ff-absdiff only supports -le and -gt".to_string()),
                },
                value,
            });
        }
    }

    for p in config.ff_reldiff_specs {
        let (left_and_right, value_part) = split_spec(&p.spec)?;
        let (left_part, right_part) = if let Some(pos) = left_and_right.rfind(':') {
            (&left_and_right[..pos], &left_and_right[pos + 1..])
        } else {
            return Err(format!(
                "missing second `:` in `{}` (expected FIELD1:FIELD2:NUM)",
                p.spec
            ));
        };
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            left_part, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            right_part, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list lengths in `{}` (left has {}, right has {})",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid numeric value `{}` in `{}`", value_part, p.spec)
        })?;
        for (l, r) in left_idxs.iter().zip(right_idxs.iter()) {
            tests.push(TestKind::FieldFieldRelDiffCmp {
                left_fields: vec![*l],
                right_fields: vec![*r],
                op: match p.op {
                    NumericOp::Le => NumericOp::Le,
                    NumericOp::Gt => NumericOp::Gt,
                    _ => return Err("ff-reldiff only supports -le and -gt".to_string()),
                },
                value,
            });
        }
    }

    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_field_reldiff_cmp_error_paths() {
        let test = TestKind::FieldFieldRelDiffCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Le,
            value: 0.1,
        };

        // Case 1: Left field missing (index out of bounds)
        // fields has 0 elements, index 1 (pos 0) is missing
        assert!(!test.eval(&[]));

        // Case 2: Left field not a number
        assert!(!test.eval(&["abc", "10.0"]));

        // Case 3: Right field missing
        // fields has 1 element, index 2 (pos 1) is missing
        assert!(!test.eval(&["10.0"]));

        // Case 4: Right field not a number
        assert!(!test.eval(&["10.0", "abc"]));
    }

    #[test]
    fn test_field_field_absdiff_cmp_error_paths() {
        let test = TestKind::FieldFieldAbsDiffCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Le,
            value: 1.0,
        };

        // Case 1: Left field missing
        assert!(!test.eval(&[]));

        // Case 2: Left field not a number
        assert!(!test.eval(&["abc", "10.0"]));

        // Case 3: Right field missing
        assert!(!test.eval(&["10.0"]));

        // Case 4: Right field not a number
        assert!(!test.eval(&["10.0", "abc"]));
    }

    #[test]
    fn test_numeric_cmp_error_paths() {
        let test = TestKind::NumericCmp {
            fields: vec![1],
            op: NumericOp::Gt,
            value: 10.0,
        };

        // Case 1: Field missing
        assert!(!test.eval(&[]));

        // Case 2: Field not a number
        assert!(!test.eval(&["abc"]));
    }

    #[test]
    fn test_numeric_prop_test_error_paths() {
        let test = TestKind::NumericPropTest {
            fields: vec![1],
            prop: NumericProp::IsNumeric,
        };

        // Case 1: Field missing
        assert!(!test.eval(&[]));

        // Case 2: Field not a number
        // IsNumeric returns true if it parses, but the implementation is:
        // match s.parse::<f64>() { Ok(_) => true, Err(_) => return false }
        // Wait, line 245: Err(_) => return false,
        // line 248: NumericProp::IsNumeric => true,
        // So if it fails to parse, it returns false.
        assert!(!test.eval(&["abc"]));
    }

    #[test]
    fn test_field_field_numeric_cmp_error_paths() {
        let test = TestKind::FieldFieldNumericCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Eq,
        };

        // Case 1: Left field missing
        assert!(!test.eval(&[]));

        // Case 2: Left field not a number
        assert!(!test.eval(&["abc", "10.0"]));

        // Case 3: Right field missing
        assert!(!test.eval(&["10.0"]));

        // Case 4: Right field not a number
        assert!(!test.eval(&["10.0", "abc"]));
    }
}
