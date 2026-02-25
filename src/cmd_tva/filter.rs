use clap::*;
use std::io::BufRead;

use crate::libs::filter::{
    build_tests, FilterSpecConfig, NumericOp, NumericProp, PendingByteLen, PendingCharLen,
    PendingFieldFieldAbsDiff, PendingFieldFieldNumeric, PendingFieldFieldRelDiff,
    PendingFieldFieldStr, PendingNumeric, PendingNumericProp, PendingRegex,
    PendingStrCmp, PendingStrEq, PendingSubstr, TestKind,
};

pub fn make_subcommand() -> Command {
    Command::new("filter")
        .about("Filters TSV rows by field-based tests")
        .after_help(include_str!("../../docs/help/filter.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV data file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first non-empty line as a header; header is always written"),
        )
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
        // Empty / blank tests
        .arg(
            Arg::new("empty")
                .long("empty")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if the field is empty (no characters)"),
        )
        .arg(
            Arg::new("not-empty")
                .long("not-empty")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if the field is not empty"),
        )
        .arg(
            Arg::new("blank")
                .long("blank")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if the field is empty or all whitespace"),
        )
        .arg(
            Arg::new("not-blank")
                .long("not-blank")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if the field contains a non-whitespace character"),
        )
        // Numeric comparisons
        .arg(
            Arg::new("gt")
                .long("gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD > NUM"),
        )
        .arg(
            Arg::new("ge")
                .long("ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD >= NUM"),
        )
        .arg(
            Arg::new("lt")
                .long("lt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD < NUM"),
        )
        .arg(
            Arg::new("le")
                .long("le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD <= NUM"),
        )
        .arg(
            Arg::new("eq")
                .long("eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD == NUM"),
        )
        .arg(
            Arg::new("ne")
                .long("ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD != NUM"),
        )
        // String comparisons
        .arg(
            Arg::new("str-gt")
                .long("str-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD > STR"),
        )
        .arg(
            Arg::new("str-ge")
                .long("str-ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD >= STR"),
        )
        .arg(
            Arg::new("str-lt")
                .long("str-lt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD < STR"),
        )
        .arg(
            Arg::new("str-le")
                .long("str-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD <= STR"),
        )
        .arg(
            Arg::new("str-eq")
                .long("str-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD == STR"),
        )
        .arg(
            Arg::new("str-ne")
                .long("str-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("String comparison: FIELD != STR"),
        )
        .arg(
            Arg::new("istr-eq")
                .long("istr-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive string comparison: FIELD == STR"),
        )
        .arg(
            Arg::new("istr-ne")
                .long("istr-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive string comparison: FIELD != STR"),
        )
        .arg(
            Arg::new("str-in-fld")
                .long("str-in-fld")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Substring test: FIELD contains STR"),
        )
        .arg(
            Arg::new("str-not-in-fld")
                .long("str-not-in-fld")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Substring test: FIELD does not contain STR"),
        )
        .arg(
            Arg::new("istr-in-fld")
                .long("istr-in-fld")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive substring test: FIELD contains STR"),
        )
        .arg(
            Arg::new("istr-not-in-fld")
                .long("istr-not-in-fld")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive substring test: FIELD does not contain STR"),
        )
        // Regex tests
        .arg(
            Arg::new("regex")
                .long("regex")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Regular expression test: FIELD matches REGEX"),
        )
        .arg(
            Arg::new("iregex")
                .long("iregex")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive regular expression test: FIELD matches REGEX"),
        )
        .arg(
            Arg::new("not-regex")
                .long("not-regex")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Regular expression test: FIELD does not match REGEX"),
        )
        .arg(
            Arg::new("not-iregex")
                .long("not-iregex")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Case-insensitive regular expression test: FIELD does not match REGEX"),
        )
        // Length tests
        .arg(
            Arg::new("char-len-gt")
                .long("char-len-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length > NUM"),
        )
        .arg(
            Arg::new("char-len-ge")
                .long("char-len-ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length >= NUM"),
        )
        .arg(
            Arg::new("char-len-lt")
                .long("char-len-lt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length < NUM"),
        )
        .arg(
            Arg::new("char-len-le")
                .long("char-len-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length <= NUM"),
        )
        .arg(
            Arg::new("char-len-eq")
                .long("char-len-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length == NUM"),
        )
        .arg(
            Arg::new("char-len-ne")
                .long("char-len-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Character length comparison: FIELD length != NUM"),
        )
        .arg(
            Arg::new("byte-len-gt")
                .long("byte-len-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length > NUM"),
        )
        .arg(
            Arg::new("byte-len-ge")
                .long("byte-len-ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length >= NUM"),
        )
        .arg(
            Arg::new("byte-len-lt")
                .long("byte-len-lt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length < NUM"),
        )
        .arg(
            Arg::new("byte-len-le")
                .long("byte-len-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length <= NUM"),
        )
        .arg(
            Arg::new("byte-len-eq")
                .long("byte-len-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length == NUM"),
        )
        .arg(
            Arg::new("byte-len-ne")
                .long("byte-len-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Byte length comparison: FIELD length != NUM"),
        )
        .arg(
            Arg::new("is-numeric")
                .long("is-numeric")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if FIELD can be parsed as a number"),
        )
        .arg(
            Arg::new("is-finite")
                .long("is-finite")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if FIELD is numeric and finite"),
        )
        .arg(
            Arg::new("is-nan")
                .long("is-nan")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if FIELD is NaN"),
        )
        .arg(
            Arg::new("is-infinity")
                .long("is-infinity")
                .num_args(1)
                .action(ArgAction::Append)
                .help("True if FIELD is positive or negative infinity"),
        )
        .arg(
            Arg::new("ff-eq")
                .long("ff-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 == FIELD2"),
        )
        .arg(
            Arg::new("ff-ne")
                .long("ff-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 != FIELD2"),
        )
        .arg(
            Arg::new("ff-lt")
                .long("ff-lt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 < FIELD2"),
        )
        .arg(
            Arg::new("ff-le")
                .long("ff-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 <= FIELD2"),
        )
        .arg(
            Arg::new("ff-gt")
                .long("ff-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 > FIELD2"),
        )
        .arg(
            Arg::new("ff-ge")
                .long("ff-ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field numeric comparison: FIELD1 >= FIELD2"),
        )
        .arg(
            Arg::new("ff-str-eq")
                .long("ff-str-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field string comparison: FIELD1 == FIELD2"),
        )
        .arg(
            Arg::new("ff-str-ne")
                .long("ff-str-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field string comparison: FIELD1 != FIELD2"),
        )
        .arg(
            Arg::new("ff-istr-eq")
                .long("ff-istr-eq")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field case-insensitive string comparison: FIELD1 == FIELD2"),
        )
        .arg(
            Arg::new("ff-istr-ne")
                .long("ff-istr-ne")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field case-insensitive string comparison: FIELD1 != FIELD2"),
        )
        .arg(
            Arg::new("ff-absdiff-le")
                .long("ff-absdiff-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field absolute difference: FIELD1:FIELD2 <= NUM"),
        )
        .arg(
            Arg::new("ff-absdiff-gt")
                .long("ff-absdiff-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field absolute difference: FIELD1:FIELD2 > NUM"),
        )
        .arg(
            Arg::new("ff-reldiff-le")
                .long("ff-reldiff-le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field relative difference: FIELD1:FIELD2 <= NUM"),
        )
        .arg(
            Arg::new("ff-reldiff-gt")
                .long("ff-reldiff-gt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Field-to-field relative difference: FIELD1:FIELD2 > NUM"),
        )
        .arg(
            Arg::new("label")
                .long("label")
                .num_args(1)
                .help("Label matched records instead of filtering; provides the header name"),
        )
        .arg(
            Arg::new("label-values")
                .long("label-values")
                .num_args(1)
                .help("Pass/No-pass values for --label, format PASS:FAIL (default 1:0)"),
        )
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva filter: {}", msg);
    std::process::exit(1);
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let has_header = args.get_flag("header");
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
                arg_error("label-values must be PASS:FAIL");
            }
        } else {
            ("1".to_string(), "0".to_string())
        }
    };
    if label_header.is_some() && count_only {
        arg_error("--label conflicts with --count");
    }

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .cloned()
        .unwrap_or_else(|| "\t".to_string());
    let mut chars = delimiter_str.chars();
    let delimiter = chars.next().unwrap_or('\t');
    if chars.next().is_some() {
        arg_error(&format!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        ));
    }

    let empty_specs: Vec<String> = args
        .get_many::<String>("empty")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let not_empty_specs: Vec<String> = args
        .get_many::<String>("not-empty")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let blank_specs: Vec<String> = args
        .get_many::<String>("blank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let not_blank_specs: Vec<String> = args
        .get_many::<String>("not-blank")
        .map(|v| v.cloned().collect())
        .unwrap_or_default();

    let numeric_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Gt,
            });
        }
        for spec in args
            .get_many::<String>("ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Ge,
            });
        }
        for spec in args
            .get_many::<String>("lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Lt,
            });
        }
        for spec in args
            .get_many::<String>("le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Eq,
            });
        }
        for spec in args
            .get_many::<String>("ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric {
                spec,
                op: NumericOp::Ne,
            });
        }
        v
    };

    let str_cmp_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("str-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrCmp {
                spec,
                op: NumericOp::Gt,
            });
        }
        for spec in args
            .get_many::<String>("str-ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrCmp {
                spec,
                op: NumericOp::Ge,
            });
        }
        for spec in args
            .get_many::<String>("str-lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrCmp {
                spec,
                op: NumericOp::Lt,
            });
        }
        for spec in args
            .get_many::<String>("str-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrCmp {
                spec,
                op: NumericOp::Le,
            });
        }
        v
    };

    let str_eq_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("str-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrEq {
                spec,
                case_insensitive: false,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("str-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrEq {
                spec,
                case_insensitive: false,
                negated: true,
            });
        }
        for spec in args
            .get_many::<String>("istr-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrEq {
                spec,
                case_insensitive: true,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("istr-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingStrEq {
                spec,
                case_insensitive: true,
                negated: true,
            });
        }
        v
    };

    let substr_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("str-in-fld")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingSubstr {
                spec,
                case_insensitive: false,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("str-not-in-fld")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingSubstr {
                spec,
                case_insensitive: false,
                negated: true,
            });
        }
        for spec in args
            .get_many::<String>("istr-in-fld")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingSubstr {
                spec,
                case_insensitive: true,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("istr-not-in-fld")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingSubstr {
                spec,
                case_insensitive: true,
                negated: true,
            });
        }
        v
    };

    let regex_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("regex")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingRegex {
                spec,
                case_insensitive: false,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("iregex")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingRegex {
                spec,
                case_insensitive: true,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("not-regex")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingRegex {
                spec,
                case_insensitive: false,
                negated: true,
            });
        }
        for spec in args
            .get_many::<String>("not-iregex")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingRegex {
                spec,
                case_insensitive: true,
                negated: true,
            });
        }
        v
    };

    let numeric_prop_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("is-numeric")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumericProp {
                spec,
                prop: NumericProp::IsNumeric,
            });
        }
        for spec in args
            .get_many::<String>("is-finite")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumericProp {
                spec,
                prop: NumericProp::IsFinite,
            });
        }
        for spec in args
            .get_many::<String>("is-nan")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumericProp {
                spec,
                prop: NumericProp::IsNaN,
            });
        }
        for spec in args
            .get_many::<String>("is-infinity")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumericProp {
                spec,
                prop: NumericProp::IsInfinity,
            });
        }
        v
    };

    let ff_numeric_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("ff-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Eq,
            });
        }
        for spec in args
            .get_many::<String>("ff-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Ne,
            });
        }
        for spec in args
            .get_many::<String>("ff-lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Lt,
            });
        }
        for spec in args
            .get_many::<String>("ff-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("ff-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Gt,
            });
        }
        for spec in args
            .get_many::<String>("ff-ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldNumeric {
                spec,
                op: NumericOp::Ge,
            });
        }
        v
    };

    let ff_str_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("ff-str-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldStr {
                spec,
                case_insensitive: false,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("ff-str-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldStr {
                spec,
                case_insensitive: false,
                negated: true,
            });
        }
        for spec in args
            .get_many::<String>("ff-istr-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldStr {
                spec,
                case_insensitive: true,
                negated: false,
            });
        }
        for spec in args
            .get_many::<String>("ff-istr-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldStr {
                spec,
                case_insensitive: true,
                negated: true,
            });
        }
        v
    };

    let ff_absdiff_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("ff-absdiff-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldAbsDiff {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("ff-absdiff-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldAbsDiff {
                spec,
                op: NumericOp::Gt,
            });
        }
        v
    };

    let ff_reldiff_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("ff-reldiff-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldRelDiff {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("ff-reldiff-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingFieldFieldRelDiff {
                spec,
                op: NumericOp::Gt,
            });
        }
        v
    };

    let char_len_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("char-len-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Gt,
            });
        }
        for spec in args
            .get_many::<String>("char-len-ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Ge,
            });
        }
        for spec in args
            .get_many::<String>("char-len-lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Lt,
            });
        }
        for spec in args
            .get_many::<String>("char-len-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("char-len-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Eq,
            });
        }
        for spec in args
            .get_many::<String>("char-len-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingCharLen {
                spec,
                op: NumericOp::Ne,
            });
        }
        v
    };

    let byte_len_specs = {
        let mut v = Vec::new();
        for spec in args
            .get_many::<String>("byte-len-gt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Gt,
            });
        }
        for spec in args
            .get_many::<String>("byte-len-ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Ge,
            });
        }
        for spec in args
            .get_many::<String>("byte-len-lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Lt,
            });
        }
        for spec in args
            .get_many::<String>("byte-len-le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Le,
            });
        }
        for spec in args
            .get_many::<String>("byte-len-eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Eq,
            });
        }
        for spec in args
            .get_many::<String>("byte-len-ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingByteLen {
                spec,
                op: NumericOp::Ne,
            });
        }
        v
    };

    let mut writer = crate::libs::io::writer("stdout");
    let mut total_matched: u64 = 0;
    let mut header_written = false;
    let mut header_seen = false;
    let mut header_struct: Option<crate::libs::fields::Header> = None;

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;

        for mut line in reader.lines().map_while(Result::ok) {
            if let Some('\r') = line.chars().last() {
                line.pop();
            }

            if has_header && !header_seen {
                if line.is_empty() {
                    continue;
                }
                header_seen = true;
                if !header_written && !count_only {
                    if let Some(ref lbl) = label_header {
                        writer
                            .write_fmt(format_args!("{}{}{}\n", line, delimiter, lbl))?;
                    } else {
                        writer.write_fmt(format_args!("{}\n", line))?;
                    }
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
                header_struct =
                    Some(crate::libs::fields::Header::from_line(&line, delimiter));
                continue;
            }

            let fields_vec: Vec<&str> = line.split(delimiter).collect();

            let header_ref = header_struct.as_ref();
            let filter_config = FilterSpecConfig {
                empty_specs: &empty_specs,
                not_empty_specs: &not_empty_specs,
                blank_specs: &blank_specs,
                not_blank_specs: &not_blank_specs,
                numeric_specs: &numeric_specs,
                str_cmp_specs: &str_cmp_specs,
                char_len_specs: &char_len_specs,
                byte_len_specs: &byte_len_specs,
                numeric_prop_specs: &numeric_prop_specs,
                str_eq_specs: &str_eq_specs,
                substr_specs: &substr_specs,
                regex_specs: &regex_specs,
                ff_numeric_specs: &ff_numeric_specs,
                ff_str_specs: &ff_str_specs,
                ff_absdiff_specs: &ff_absdiff_specs,
                ff_reldiff_specs: &ff_reldiff_specs,
            };

            let tests: Vec<TestKind> = build_tests(header_ref, delimiter, filter_config)
                .unwrap_or_else(|e| arg_error(&e));

            let mut row_match = if tests.is_empty() {
                true
            } else if use_or {
                let mut any = false;
                for t in &tests {
                    if t.eval(&fields_vec) {
                        any = true;
                        break;
                    }
                }
                any
            } else {
                let mut all = true;
                for t in &tests {
                    if !t.eval(&fields_vec) {
                        all = false;
                        break;
                    }
                }
                all
            };

            if invert {
                row_match = !row_match;
            }

            if label_header.is_some() {
                let val = if row_match {
                    &label_pass_val
                } else {
                    &label_fail_val
                };
                writer.write_fmt(format_args!("{}{}{}\n", line, delimiter, val))?;
                if line_buffered {
                    writer.flush()?;
                }
            } else if row_match {
                if count_only {
                    total_matched += 1;
                } else {
                    writer.write_fmt(format_args!("{}\n", line))?;
                    if line_buffered {
                        writer.flush()?;
                    }
                }
            }
        }
    }

    if count_only {
        println!("{}", total_matched);
    }

    Ok(())
}
