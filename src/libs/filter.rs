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
use unicode_segmentation::UnicodeSegmentation;

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
            TestKind::CharLenCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
                    None => "",
                };
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
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
                    None => "",
                };
                let len = s.len() as f64;
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
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
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
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                match fields.get(pos) {
                    Some(s) => {
                        if *case_insensitive {
                            s.to_lowercase() == value.to_lowercase()
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
                            s.to_lowercase() != value.to_lowercase()
                        } else {
                            *s != value
                        }
                    }
                    None => true,
                }
            }),
            TestKind::StrCmp {
                fields: idxs,
                op,
                value,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                match fields.get(pos) {
                    Some(s) => match op {
                        NumericOp::Gt => *s > value.as_str(),
                        NumericOp::Ge => *s >= value.as_str(),
                        NumericOp::Lt => *s < value.as_str(),
                        NumericOp::Le => *s <= value.as_str(),
                        NumericOp::Eq => *s == value.as_str(),
                        NumericOp::Ne => *s != value.as_str(),
                    },
                    None => false,
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
            TestKind::Regex {
                fields: idxs,
                regex,
                negated,
            } => idxs.iter().all(|idx| {
                let pos = idx.saturating_sub(1);
                let s = match fields.get(pos) {
                    Some(v) => *v,
                    None => "",
                };
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
                    let l_pos = l.saturating_sub(1);
                    let l_s = match fields.get(l_pos) {
                        Some(v) => *v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_pos = r.saturating_sub(1);
                        let r_s = match fields.get(r_pos) {
                            Some(v) => *v,
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
                    let l_pos = l.saturating_sub(1);
                    let r_pos = r.saturating_sub(1);
                    let l_s = match fields.get(l_pos) {
                        Some(v) => *v,
                        None => "",
                    };
                    let r_s = match fields.get(r_pos) {
                        Some(v) => *v,
                        None => "",
                    };
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
                    let l_pos = l.saturating_sub(1);
                    let l_s = match fields.get(l_pos) {
                        Some(v) => *v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_pos = r.saturating_sub(1);
                        let r_s = match fields.get(r_pos) {
                            Some(v) => *v,
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
                    let l_pos = l.saturating_sub(1);
                    let l_s = match fields.get(l_pos) {
                        Some(v) => *v,
                        None => return false,
                    };
                    let l_v = match l_s.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        let r_pos = r.saturating_sub(1);
                        let r_s = match fields.get(r_pos) {
                            Some(v) => *v,
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
        let idxs =
            crate::libs::tsv::fields::parse_field_list_with_header(spec, header, delimiter)?;
        for idx in idxs {
            tests.push(TestKind::Empty { fields: vec![idx] });
        }
    }

    for spec in config.not_empty_specs {
        let idxs =
            crate::libs::tsv::fields::parse_field_list_with_header(spec, header, delimiter)?;
        for idx in idxs {
            tests.push(TestKind::NotEmpty { fields: vec![idx] });
        }
    }

    for spec in config.blank_specs {
        let idxs =
            crate::libs::tsv::fields::parse_field_list_with_header(spec, header, delimiter)?;
        for idx in idxs {
            tests.push(TestKind::Blank { fields: vec![idx] });
        }
    }

    for spec in config.not_blank_specs {
        let idxs =
            crate::libs::tsv::fields::parse_field_list_with_header(spec, header, delimiter)?;
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
