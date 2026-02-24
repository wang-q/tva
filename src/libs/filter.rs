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
//!
//! let row = ["id", "10.5"];
//! let test = TestKind::NumericCmp {
//!     fields: vec![2],
//!     op: NumericOp::Gt,
//!     value: 10.0,
//! };
//!
//! assert!(test.eval(&row));
//! ```
//!
//! Substring matching on a single row:
//!
//! ```
//! use tva::libs::filter::TestKind;
//!
//! let row = ["foo", "barbaz"];
//! let test = TestKind::StrIn {
//!     fields: vec![2],
//!     value: "bar".to_string(),
//!     case_insensitive: false,
//!     negated: false,
//! };
//!
//! assert!(test.eval(&row));
//! ```

use regex::Regex;

pub enum NumericOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

pub enum TestKind {
    Empty { fields: Vec<usize> },
    NotEmpty { fields: Vec<usize> },
    Blank { fields: Vec<usize> },
    NotBlank { fields: Vec<usize> },
    NumericCmp {
        fields: Vec<usize>,
        op: NumericOp,
        value: f64,
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
    StrIn {
        fields: Vec<usize>,
        value: String,
        case_insensitive: bool,
        negated: bool,
    },
    Regex {
        fields: Vec<usize>,
        regex: Regex,
    },
}

impl TestKind {
    pub fn eval(&self, fields: &[&str]) -> bool {
        match self {
            TestKind::Empty { fields: idxs } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                fields.get(pos).map(|s| s.is_empty()).unwrap_or(true)
            }),
            TestKind::NotEmpty { fields: idxs } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                fields.get(pos).map(|s| !s.is_empty()).unwrap_or(false)
            }),
            TestKind::Blank { fields: idxs } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                fields
                    .get(pos)
                    .map(|s| s.chars().all(|c| c.is_whitespace()))
                    .unwrap_or(true)
            }),
            TestKind::NotBlank { fields: idxs } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                fields
                    .get(pos)
                    .map(|s| s.chars().any(|c| !c.is_whitespace()))
                    .unwrap_or(false)
            }),
            TestKind::NumericCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
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
            TestKind::StrEq {
                fields: idxs,
                value,
                case_insensitive,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                match fields.get(pos) {
                    Some(s) => {
                        if *case_insensitive {
                            s.eq_ignore_ascii_case(value)
                        } else {
                            *s == value
                        }
                    }
                    None => false,
                }
            }),
            TestKind::StrNe {
                fields: idxs,
                value,
                case_insensitive,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                match fields.get(pos) {
                    Some(s) => {
                        if *case_insensitive {
                            !s.eq_ignore_ascii_case(value)
                        } else {
                            *s != value
                        }
                    }
                    None => true,
                }
            }),
            TestKind::StrIn {
                fields: idxs,
                value,
                case_insensitive,
                negated,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                let haystack = match fields.get(pos) {
                    Some(s) => *s,
                    None => "",
                };
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
            TestKind::Regex { fields: idxs, regex } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
                    None => "",
                };
                regex.is_match(s)
            }),
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

pub struct PendingSubstr {
    pub spec: String,
    pub case_insensitive: bool,
    pub negated: bool,
}

pub struct PendingRegex {
    pub spec: String,
    pub case_insensitive: bool,
}

pub fn split_spec(spec: &str) -> Result<(String, String), String> {
    if let Some(pos) = spec.rfind(':') {
        let (left, right) = spec.split_at(pos);
        Ok((left.to_string(), right[1..].to_string()))
    } else {
        Err(format!(
            "missing `:` separator in `{}` (expected <field-list>:<value>)",
            spec
        ))
    }
}

pub fn build_tests(
    header: Option<&crate::libs::fields::Header>,
    delimiter: char,
    empty_specs: &[String],
    not_empty_specs: &[String],
    blank_specs: &[String],
    not_blank_specs: &[String],
    numeric_specs: &[PendingNumeric],
    str_eq_specs: &[PendingStrEq],
    substr_specs: &[PendingSubstr],
    regex_specs: &[PendingRegex],
) -> Result<Vec<TestKind>, String> {
    let mut tests = Vec::new();

    for spec in empty_specs {
        let idxs = crate::libs::fields::parse_field_list_with_header(
            spec,
            header,
            delimiter,
        )?;
        tests.push(TestKind::Empty { fields: idxs });
    }

    for spec in not_empty_specs {
        let idxs = crate::libs::fields::parse_field_list_with_header(
            spec,
            header,
            delimiter,
        )?;
        tests.push(TestKind::NotEmpty { fields: idxs });
    }

    for spec in blank_specs {
        let idxs = crate::libs::fields::parse_field_list_with_header(
            spec,
            header,
            delimiter,
        )?;
        tests.push(TestKind::Blank { fields: idxs });
    }

    for spec in not_blank_specs {
        let idxs = crate::libs::fields::parse_field_list_with_header(
            spec,
            header,
            delimiter,
        )?;
        tests.push(TestKind::NotBlank { fields: idxs });
    }

    for p in numeric_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid numeric value `{}` in `{}`", value_part, p.spec)
        })?;
        tests.push(TestKind::NumericCmp {
            fields: idxs,
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

    for p in str_eq_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        if p.negated {
            tests.push(TestKind::StrNe {
                fields: idxs,
                value: value_part.to_string(),
                case_insensitive: p.case_insensitive,
            });
        } else {
            tests.push(TestKind::StrEq {
                fields: idxs,
                value: value_part.to_string(),
                case_insensitive: p.case_insensitive,
            });
        }
    }

    for p in substr_specs {
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        tests.push(TestKind::StrIn {
            fields: idxs,
            value: value_part.to_string(),
            case_insensitive: p.case_insensitive,
            negated: p.negated,
        });
    }

    for p in regex_specs {
        let (field_part, pattern) = split_spec(&p.spec)?;
        let idxs = crate::libs::fields::parse_field_list_with_header(
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
        tests.push(TestKind::Regex { fields: idxs, regex });
    }

    Ok(tests)
}
