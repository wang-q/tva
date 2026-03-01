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
