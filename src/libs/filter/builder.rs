use super::config::{FilterSpecConfig, NumericOp, NumericProp};
use super::engine::TestKind;

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
        for idx in idxs {
            if p.negated {
                tests.push(TestKind::StrNe {
                    fields: vec![idx],
                    value: value_part.clone(),
                    case_insensitive: p.case_insensitive,
                });
            } else {
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
        let (field_part, value_part) = split_spec(&p.spec)?;
        let idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &field_part,
            header,
            delimiter,
        )?;
        let regex = if p.case_insensitive {
            regex::RegexBuilder::new(&value_part)
                .case_insensitive(true)
                .build()
        } else {
            regex::Regex::new(&value_part)
        };
        let regex = regex.map_err(|e| format!("invalid regex `{}`: {}", value_part, e))?;

        for idx in idxs {
            tests.push(TestKind::Regex {
                fields: vec![idx],
                regex: regex.clone(),
                negated: p.negated,
            });
        }
    }

    // Field-Field comparisons
    for p in config.ff_numeric_specs {
        // spec is "FIELD1:FIELD2"
        let (left, right) = split_spec(&p.spec)?;
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list in numeric comparison `{}`: left {} fields, right {} fields",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        tests.push(TestKind::FieldFieldNumericCmp {
            left_fields: left_idxs,
            right_fields: right_idxs,
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

    for p in config.ff_str_specs {
        let (left, right) = split_spec(&p.spec)?;
        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list in string comparison `{}`: left {} fields, right {} fields",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        tests.push(TestKind::FieldFieldStrCmp {
            left_fields: left_idxs,
            right_fields: right_idxs,
            case_insensitive: p.case_insensitive,
            negated: p.negated,
        });
    }

    for p in config.ff_absdiff_specs {
        // spec is "FIELD1:FIELD2:NUM"
        // We need to split twice from right
        let (rest, value_part) = split_spec(&p.spec)?;
        let (left, right) = split_spec(&rest).map_err(|_| {
            format!(
                "missing second `:` in absdiff spec `{}` (expected FIELD1:FIELD2:NUM)",
                p.spec
            )
        })?;

        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid diff value `{}` in `{}`", value_part, p.spec)
        })?;

        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list in absdiff comparison `{}`: left {} fields, right {} fields",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        tests.push(TestKind::FieldFieldAbsDiffCmp {
            left_fields: left_idxs,
            right_fields: right_idxs,
            op: match p.op {
                NumericOp::Gt => NumericOp::Gt,
                NumericOp::Le => NumericOp::Le,
                _ => return Err(format!("unsupported op for absdiff in `{}`", p.spec)),
            },
            value,
        });
    }

    for p in config.ff_reldiff_specs {
        // spec is "FIELD1:FIELD2:NUM"
        let (rest, value_part) = split_spec(&p.spec)?;
        let (left, right) = split_spec(&rest).map_err(|_| {
            format!(
                "missing second `:` in reldiff spec `{}` (expected FIELD1:FIELD2:NUM)",
                p.spec
            )
        })?;

        let value = value_part.parse::<f64>().map_err(|_| {
            format!("invalid diff value `{}` in `{}`", value_part, p.spec)
        })?;

        let left_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &left, header, delimiter,
        )?;
        let right_idxs = crate::libs::tsv::fields::parse_field_list_with_header(
            &right, header, delimiter,
        )?;
        if left_idxs.len() != right_idxs.len() {
            return Err(format!(
                "mismatched field list in reldiff comparison `{}`: left {} fields, right {} fields",
                p.spec,
                left_idxs.len(),
                right_idxs.len()
            ));
        }
        tests.push(TestKind::FieldFieldRelDiffCmp {
            left_fields: left_idxs,
            right_fields: right_idxs,
            op: match p.op {
                NumericOp::Gt => NumericOp::Gt,
                NumericOp::Le => NumericOp::Le,
                _ => return Err(format!("unsupported op for reldiff in `{}`", p.spec)),
            },
            value,
        });
    }

    Ok(tests)
}
