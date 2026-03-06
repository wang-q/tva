use super::config::{NumericOp, NumericProp};
use crate::libs::number::fast_parse_f64;
use crate::libs::tsv::record::{Row, StrSliceRow};
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
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
                let bytes = match row.get_bytes(*idx) {
                    Some(v) => v,
                    None => return false,
                };
                let parsed = match fast_parse_f64(bytes) {
                    Some(v) => v,
                    None => return false,
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
                let bytes = match row.get_bytes(*idx) {
                    Some(v) => v,
                    None => return false,
                };
                let parsed = match fast_parse_f64(bytes) {
                    Some(v) => v,
                    None => return false,
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
                ..
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
                ..
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
                    let l_v = match row.get_bytes(*l).and_then(fast_parse_f64) {
                        Some(v) => v,
                        None => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        match row.get_bytes(*r).and_then(fast_parse_f64) {
                            Some(v) => v,
                            None => return false,
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
                    let l_v = match row.get_bytes(*l).and_then(fast_parse_f64) {
                        Some(v) => v,
                        None => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        match row.get_bytes(*r).and_then(fast_parse_f64) {
                            Some(v) => v,
                            None => return false,
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
                    let l_v = match row.get_bytes(*l).and_then(fast_parse_f64) {
                        Some(v) => v,
                        None => return false,
                    };
                    let r_v = if l == r {
                        l_v
                    } else {
                        match row.get_bytes(*r).and_then(fast_parse_f64) {
                            Some(v) => v,
                            None => return false,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::filter::config::NumericOp;
    use crate::libs::tsv::record::StrSliceRow;

    // Helper to create a row
    fn row<'a>(fields: &'a [&'a str]) -> StrSliceRow<'a> {
        StrSliceRow { fields }
    }

    #[test]
    fn test_numeric_cmp_edge_cases() {
        // Invalid number string
        let test = TestKind::NumericCmp {
            fields: vec![1], // 1-based index (field 1)
            op: NumericOp::Gt,
            value: 10.0,
        };
        assert!(
            !test.eval_row(&row(&["abc"])),
            "Should return false for invalid number"
        );

        // Missing field (index 2 doesn't exist)
        let test = TestKind::NumericCmp {
            fields: vec![2],
            op: NumericOp::Gt,
            value: 10.0,
        };
        assert!(
            !test.eval_row(&row(&["15.0"])),
            "Should return false for missing field"
        );
    }

    #[test]
    fn test_str_eq_case_insensitive() {
        let test = TestKind::StrEq {
            fields: vec![1],
            value: "FOO".to_string(),
            case_insensitive: true,
        };
        assert!(test.eval_row(&row(&["foo"])));
        assert!(test.eval_row(&row(&["FOO"])));

        let test_sensitive = TestKind::StrEq {
            fields: vec![1],
            value: "FOO".to_string(),
            case_insensitive: false,
        };
        assert!(!test_sensitive.eval_row(&row(&["foo"])));
        assert!(test_sensitive.eval_row(&row(&["FOO"])));
    }

    #[test]
    fn test_str_ne_missing() {
        let test = TestKind::StrNe {
            fields: vec![2], // Missing field
            value: "foo".to_string(),
            case_insensitive: false,
        };
        // None branch returns true (if field is missing, it's not equal to value)
        assert!(test.eval_row(&row(&["bar"])));
    }

    #[test]
    fn test_ff_numeric_cmp_edge_cases() {
        // Mismatched lengths
        let test = TestKind::FieldFieldNumericCmp {
            left_fields: vec![1],
            right_fields: vec![1, 2],
            op: NumericOp::Eq,
        };
        assert!(
            !test.eval_row(&row(&["1", "1"])),
            "Should return false for mismatched field count"
        );

        // Same field optimization (l == r)
        let test = TestKind::FieldFieldNumericCmp {
            left_fields: vec![1],
            right_fields: vec![1],
            op: NumericOp::Eq,
        };
        assert!(test.eval_row(&row(&["10"])), "Should match same field");

        // Invalid number in right field
        let test = TestKind::FieldFieldNumericCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Eq,
        };
        assert!(
            !test.eval_row(&row(&["10", "abc"])),
            "Should return false if parsing fails"
        );
    }

    #[test]
    fn test_ff_str_cmp_optimization() {
        // l == r optimization
        let test = TestKind::FieldFieldStrCmp {
            left_fields: vec![1],
            right_fields: vec![1],
            case_insensitive: false,
            negated: false,
        };
        assert!(test.eval_row(&row(&["foo"])));

        let test_neg = TestKind::FieldFieldStrCmp {
            left_fields: vec![1],
            right_fields: vec![1],
            case_insensitive: false,
            negated: true,
        };
        assert!(!test_neg.eval_row(&row(&["foo"])));
    }

    #[test]
    fn test_ff_reldiff_edge_cases() {
        // Same field (diff = 0)
        let test = TestKind::FieldFieldRelDiffCmp {
            left_fields: vec![1],
            right_fields: vec![1],
            op: NumericOp::Le,
            value: 0.0,
        };
        assert!(test.eval_row(&row(&["10"])));

        // Denom = 0 (infinity case)
        // left=10, right=0. diff=10. denom=min(|10|, |0|) = 0. rel=inf.
        let test = TestKind::FieldFieldRelDiffCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Gt,
            value: 1000.0, // inf > 1000
        };
        assert!(test.eval_row(&row(&["10", "0"])));

        // Unsupported op
        let test = TestKind::FieldFieldRelDiffCmp {
            left_fields: vec![1],
            right_fields: vec![2],
            op: NumericOp::Eq, // Not supported in match -> false
            value: 0.1,
        };
        assert!(!test.eval_row(&row(&["10", "11"])));
    }

    #[test]
    fn test_regex_negated() {
        let test = TestKind::Regex {
            fields: vec![1],
            regex: regex::Regex::new("foo").unwrap(),
            negated: true,
        };
        assert!(test.eval_row(&row(&["bar"])));
        assert!(!test.eval_row(&row(&["foobar"])));
    }

    #[test]
    fn test_max_field_index() {
        let test = TestKind::NumericCmp {
            fields: vec![1, 5],
            op: NumericOp::Gt,
            value: 10.0,
        };
        assert_eq!(test.max_field_index(), 5);

        let test_ff = TestKind::FieldFieldNumericCmp {
            left_fields: vec![1],
            right_fields: vec![3],
            op: NumericOp::Eq,
        };
        assert_eq!(test_ff.max_field_index(), 3);

        let test_empty = TestKind::Empty { fields: vec![] };
        assert_eq!(test_empty.max_field_index(), 0);
    }
}
