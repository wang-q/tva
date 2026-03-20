use clap::*;

use crate::libs::cli::{build_header_config, get_delimiter, header_args_with_columns};
use crate::libs::filter::{
    FilterConfig, NumericOp, NumericProp, PendingByteLen, PendingCharLen,
    PendingFieldFieldAbsDiff, PendingFieldFieldNumeric, PendingFieldFieldRelDiff,
    PendingFieldFieldStr, PendingNumeric, PendingNumericProp, PendingRegex,
    PendingStrCmp, PendingStrEq, PendingSubstr,
};

pub fn make_subcommand() -> Command {
    let mut cmd = Command::new("filter")
        .about("Filters TSV rows by field-based tests")
        .after_help(include_str!("../../docs/help/filter.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV data file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter character (default: TAB)"),
        )
        .arg(
            Arg::new("or")
                .long("or")
                .action(ArgAction::SetTrue)
                .help("Evaluate tests as OR instead of AND"),
        )
        .arg(
            Arg::new("invert")
                .long("invert")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Invert the filter, selecting non-matching rows"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Print only the count of matching data rows"),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Enable line-buffered output (flush after each line)"),
        )
        .arg(Arg::new("label").long("label").num_args(1).help(
            "Label matched records instead of filtering; provides the header name",
        ))
        .arg(
            Arg::new("label-values")
                .long("label-values")
                .num_args(1)
                .help("Pass/No-pass values for --label, format PASS:FAIL (default 1:0)"),
        )
        .arg(
            Arg::new("expr")
                .long("expr")
                .short('E')
                .num_args(1)
                .help("Filter expression (e.g., '@price > 100 and @stock > 0')"),
        );

    macro_rules! arg_test {
        ($name:expr, $help:expr) => {
            cmd = cmd.arg(
                Arg::new($name)
                    .long($name)
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help($help),
            );
        };
    }

    // Empty / blank tests
    arg_test!("empty", "True if the field is empty (no characters)");
    arg_test!("not-empty", "True if the field is not empty");
    arg_test!("blank", "True if the field is empty or all whitespace");
    arg_test!(
        "not-blank",
        "True if the field contains a non-whitespace character"
    );

    // Numeric comparisons
    arg_test!("gt", "Numeric comparison: FIELD > NUM");
    arg_test!("ge", "Numeric comparison: FIELD >= NUM");
    arg_test!("lt", "Numeric comparison: FIELD < NUM");
    arg_test!("le", "Numeric comparison: FIELD <= NUM");
    arg_test!("eq", "Numeric comparison: FIELD == NUM");
    arg_test!("ne", "Numeric comparison: FIELD != NUM");

    // String comparisons
    arg_test!("str-gt", "String comparison: FIELD > STR");
    arg_test!("str-ge", "String comparison: FIELD >= STR");
    arg_test!("str-lt", "String comparison: FIELD < STR");
    arg_test!("str-le", "String comparison: FIELD <= STR");
    arg_test!("str-eq", "String comparison: FIELD == STR");
    arg_test!("str-ne", "String comparison: FIELD != STR");
    arg_test!(
        "istr-eq",
        "Case-insensitive string comparison: FIELD == STR"
    );
    arg_test!(
        "istr-ne",
        "Case-insensitive string comparison: FIELD != STR"
    );

    // Substring tests
    arg_test!("str-in-fld", "Substring test: FIELD contains STR");
    arg_test!(
        "str-not-in-fld",
        "Substring test: FIELD does not contain STR"
    );
    arg_test!(
        "istr-in-fld",
        "Case-insensitive substring test: FIELD contains STR"
    );
    arg_test!(
        "istr-not-in-fld",
        "Case-insensitive substring test: FIELD does not contain STR"
    );

    // Regex tests
    arg_test!("regex", "Regular expression test: FIELD matches REGEX");
    arg_test!(
        "iregex",
        "Case-insensitive regular expression test: FIELD matches REGEX"
    );
    arg_test!(
        "not-regex",
        "Regular expression test: FIELD does not match REGEX"
    );
    arg_test!(
        "not-iregex",
        "Case-insensitive regular expression test: FIELD does not match REGEX"
    );

    // Length tests
    arg_test!(
        "char-len-gt",
        "Character length comparison: FIELD length > NUM"
    );
    arg_test!(
        "char-len-ge",
        "Character length comparison: FIELD length >= NUM"
    );
    arg_test!(
        "char-len-lt",
        "Character length comparison: FIELD length < NUM"
    );
    arg_test!(
        "char-len-le",
        "Character length comparison: FIELD length <= NUM"
    );
    arg_test!(
        "char-len-eq",
        "Character length comparison: FIELD length == NUM"
    );
    arg_test!(
        "char-len-ne",
        "Character length comparison: FIELD length != NUM"
    );

    arg_test!("byte-len-gt", "Byte length comparison: FIELD length > NUM");
    arg_test!("byte-len-ge", "Byte length comparison: FIELD length >= NUM");
    arg_test!("byte-len-lt", "Byte length comparison: FIELD length < NUM");
    arg_test!("byte-len-le", "Byte length comparison: FIELD length <= NUM");
    arg_test!("byte-len-eq", "Byte length comparison: FIELD length == NUM");
    arg_test!("byte-len-ne", "Byte length comparison: FIELD length != NUM");

    // Numeric properties
    arg_test!("is-numeric", "True if FIELD can be parsed as a number");
    arg_test!("is-finite", "True if FIELD is numeric and finite");
    arg_test!("is-nan", "True if FIELD is NaN");
    arg_test!(
        "is-infinity",
        "True if FIELD is positive or negative infinity"
    );

    // Field-Field comparisons
    arg_test!(
        "ff-eq",
        "Field-to-field numeric comparison: FIELD1 == FIELD2"
    );
    arg_test!(
        "ff-ne",
        "Field-to-field numeric comparison: FIELD1 != FIELD2"
    );
    arg_test!(
        "ff-lt",
        "Field-to-field numeric comparison: FIELD1 < FIELD2"
    );
    arg_test!(
        "ff-le",
        "Field-to-field numeric comparison: FIELD1 <= FIELD2"
    );
    arg_test!(
        "ff-gt",
        "Field-to-field numeric comparison: FIELD1 > FIELD2"
    );
    arg_test!(
        "ff-ge",
        "Field-to-field numeric comparison: FIELD1 >= FIELD2"
    );

    arg_test!(
        "ff-str-eq",
        "Field-to-field string comparison: FIELD1 == FIELD2"
    );
    arg_test!(
        "ff-str-ne",
        "Field-to-field string comparison: FIELD1 != FIELD2"
    );
    arg_test!(
        "ff-istr-eq",
        "Field-to-field case-insensitive string comparison: FIELD1 == FIELD2"
    );
    arg_test!(
        "ff-istr-ne",
        "Field-to-field case-insensitive string comparison: FIELD1 != FIELD2"
    );

    arg_test!(
        "ff-absdiff-le",
        "Field-to-field absolute difference: FIELD1:FIELD2 <= NUM"
    );
    arg_test!(
        "ff-absdiff-gt",
        "Field-to-field absolute difference: FIELD1:FIELD2 > NUM"
    );

    arg_test!(
        "ff-reldiff-le",
        "Field-to-field relative difference: FIELD1:FIELD2 <= NUM"
    );
    arg_test!(
        "ff-reldiff-gt",
        "Field-to-field relative difference: FIELD1:FIELD2 > NUM"
    );

    cmd
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let use_or = args.get_flag("or");
    let invert = args.get_flag("invert");
    let count_only = args.get_flag("count");
    let line_buffered = args.get_flag("line-buffered");
    let label_header: Option<String> = args.get_one::<String>("label").cloned();
    let (label_pass_val, label_fail_val) = {
        if let Some(v) = args.get_one::<String>("label-values") {
            if let Some(pos) = v.rfind(':') {
                (v[..pos].to_string(), v[pos + 1..].to_string())
            } else {
                anyhow::bail!("label-values must be PASS:FAIL");
            }
        } else {
            ("1".to_string(), "0".to_string())
        }
    };
    if label_header.is_some() && count_only {
        anyhow::bail!("--label conflicts with --count");
    }

    let opt_delimiter = get_delimiter(args, "delimiter")? as char;

    // Helper macro for simple string lists
    macro_rules! collect_simple {
        ($name:expr) => {
            args.get_many::<String>($name)
                .map(|v| v.cloned().collect())
                .unwrap_or_default()
        };
    }

    let empty_specs: Vec<String> = collect_simple!("empty");
    let not_empty_specs: Vec<String> = collect_simple!("not-empty");
    let blank_specs: Vec<String> = collect_simple!("blank");
    let not_blank_specs: Vec<String> = collect_simple!("not-blank");

    // Helper macro for typed specs with an operator/property field
    macro_rules! collect_typed {
        ($type:ident, $field:ident, $(($name:expr, $val:expr)),* ) => {
            {
                let mut v = Vec::new();
                $(
                    if let Some(values) = args.get_many::<String>($name) {
                        for spec in values {
                            v.push($type {
                                spec: spec.clone(),
                                $field: $val,
                            });
                        }
                    }
                )*
                v
            }
        };
    }

    // Helper macro for specs with case_insensitive and negated flags
    macro_rules! collect_flags {
        ($type:ident, $(($name:expr, $case:expr, $neg:expr)),* ) => {
            {
                let mut v = Vec::new();
                $(
                    if let Some(values) = args.get_many::<String>($name) {
                        for spec in values {
                            v.push($type {
                                spec: spec.clone(),
                                case_insensitive: $case,
                                negated: $neg,
                            });
                        }
                    }
                )*
                v
            }
        };
    }

    let numeric_specs = collect_typed!(
        PendingNumeric,
        op,
        ("gt", NumericOp::Gt),
        ("ge", NumericOp::Ge),
        ("lt", NumericOp::Lt),
        ("le", NumericOp::Le),
        ("eq", NumericOp::Eq),
        ("ne", NumericOp::Ne)
    );

    let str_cmp_specs = collect_typed!(
        PendingStrCmp,
        op,
        ("str-gt", NumericOp::Gt),
        ("str-ge", NumericOp::Ge),
        ("str-lt", NumericOp::Lt),
        ("str-le", NumericOp::Le)
    );

    let str_eq_specs = collect_flags!(
        PendingStrEq,
        ("str-eq", false, false),
        ("str-ne", false, true),
        ("istr-eq", true, false),
        ("istr-ne", true, true)
    );

    let substr_specs = collect_flags!(
        PendingSubstr,
        ("str-in-fld", false, false),
        ("str-not-in-fld", false, true),
        ("istr-in-fld", true, false),
        ("istr-not-in-fld", true, true)
    );

    let regex_specs = collect_flags!(
        PendingRegex,
        ("regex", false, false),
        ("iregex", true, false),
        ("not-regex", false, true),
        ("not-iregex", true, true)
    );

    let numeric_prop_specs = collect_typed!(
        PendingNumericProp,
        prop,
        ("is-numeric", NumericProp::IsNumeric),
        ("is-finite", NumericProp::IsFinite),
        ("is-nan", NumericProp::IsNaN),
        ("is-infinity", NumericProp::IsInfinity)
    );

    let ff_numeric_specs = collect_typed!(
        PendingFieldFieldNumeric,
        op,
        ("ff-eq", NumericOp::Eq),
        ("ff-ne", NumericOp::Ne),
        ("ff-lt", NumericOp::Lt),
        ("ff-le", NumericOp::Le),
        ("ff-gt", NumericOp::Gt),
        ("ff-ge", NumericOp::Ge)
    );

    let ff_str_specs = collect_flags!(
        PendingFieldFieldStr,
        ("ff-str-eq", false, false),
        ("ff-str-ne", false, true),
        ("ff-istr-eq", true, false),
        ("ff-istr-ne", true, true)
    );

    let ff_absdiff_specs = collect_typed!(
        PendingFieldFieldAbsDiff,
        op,
        ("ff-absdiff-le", NumericOp::Le),
        ("ff-absdiff-gt", NumericOp::Gt)
    );

    let ff_reldiff_specs = collect_typed!(
        PendingFieldFieldRelDiff,
        op,
        ("ff-reldiff-le", NumericOp::Le),
        ("ff-reldiff-gt", NumericOp::Gt)
    );

    let char_len_specs = collect_typed!(
        PendingCharLen,
        op,
        ("char-len-gt", NumericOp::Gt),
        ("char-len-ge", NumericOp::Ge),
        ("char-len-lt", NumericOp::Lt),
        ("char-len-le", NumericOp::Le),
        ("char-len-eq", NumericOp::Eq),
        ("char-len-ne", NumericOp::Ne)
    );

    let byte_len_specs = collect_typed!(
        PendingByteLen,
        op,
        ("byte-len-gt", NumericOp::Gt),
        ("byte-len-ge", NumericOp::Ge),
        ("byte-len-lt", NumericOp::Lt),
        ("byte-len-le", NumericOp::Le),
        ("byte-len-eq", NumericOp::Eq),
        ("byte-len-ne", NumericOp::Ne)
    );

    let config = FilterConfig {
        delimiter: opt_delimiter,
        header_config,
        use_or,
        invert,
        count_only,
        line_buffered,
        label_header,
        label_pass_val,
        label_fail_val,
        empty_specs,
        not_empty_specs,
        blank_specs,
        not_blank_specs,
        numeric_specs,
        str_cmp_specs,
        char_len_specs,
        byte_len_specs,
        numeric_prop_specs,
        str_eq_specs,
        substr_specs,
        regex_specs,
        ff_numeric_specs,
        ff_str_specs,
        ff_absdiff_specs,
        ff_reldiff_specs,
    };

    crate::libs::filter::run_filter(&infiles, &mut writer, config)
        .map_err(|e| anyhow::anyhow!("tva filter: {}", e))?;

    Ok(())
}
