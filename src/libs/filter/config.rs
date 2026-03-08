use crate::libs::tsv::header::HeaderConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericProp {
    IsNumeric,
    IsFinite,
    IsNaN,
    IsInfinity,
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

#[derive(Default)]
pub struct FilterConfig {
    pub delimiter: char,
    pub header_config: HeaderConfig,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config_default() {
        let config = FilterConfig::default();

        assert_eq!(config.delimiter, '\0');
        assert_eq!(config.header_config.enabled, false);
        assert_eq!(config.use_or, false);
        assert_eq!(config.invert, false);
        assert_eq!(config.count_only, false);
        assert_eq!(config.line_buffered, false);
        assert!(config.label_header.is_none());
        assert_eq!(config.label_pass_val, "");
        assert_eq!(config.label_fail_val, "");

        assert!(config.empty_specs.is_empty());
        assert!(config.not_empty_specs.is_empty());
        assert!(config.blank_specs.is_empty());
        assert!(config.not_blank_specs.is_empty());
        assert!(config.numeric_specs.is_empty());
        assert!(config.str_cmp_specs.is_empty());
        assert!(config.char_len_specs.is_empty());
        assert!(config.byte_len_specs.is_empty());
        assert!(config.numeric_prop_specs.is_empty());
        assert!(config.str_eq_specs.is_empty());
        assert!(config.substr_specs.is_empty());
        assert!(config.regex_specs.is_empty());
        assert!(config.ff_numeric_specs.is_empty());
        assert!(config.ff_str_specs.is_empty());
        assert!(config.ff_absdiff_specs.is_empty());
        assert!(config.ff_reldiff_specs.is_empty());
    }

    #[test]
    fn test_as_spec_config() {
        let mut config = FilterConfig::default();
        config.empty_specs.push("1".to_string());
        config.numeric_specs.push(PendingNumeric {
            spec: "1:10".to_string(),
            op: NumericOp::Gt,
        });
        config.str_cmp_specs.push(PendingStrCmp {
            spec: "2:foo".to_string(),
            op: NumericOp::Eq,
        });

        let spec_config = config.as_spec_config();

        assert_eq!(spec_config.empty_specs.len(), 1);
        assert_eq!(spec_config.empty_specs[0], "1");

        assert_eq!(spec_config.numeric_specs.len(), 1);
        assert_eq!(spec_config.numeric_specs[0].spec, "1:10");
        assert_eq!(spec_config.numeric_specs[0].op, NumericOp::Gt);

        assert_eq!(spec_config.str_cmp_specs.len(), 1);
        assert_eq!(spec_config.str_cmp_specs[0].spec, "2:foo");
        assert_eq!(spec_config.str_cmp_specs[0].op, NumericOp::Eq);

        // Check other fields are still empty
        assert!(spec_config.not_empty_specs.is_empty());
        assert!(spec_config.blank_specs.is_empty());
    }
}
