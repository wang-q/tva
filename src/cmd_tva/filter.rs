use clap::*;
use regex::Regex;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("filter")
        .about("Filters TSV rows by field-based tests")
        .after_help(
            r###"
Filters tab-separated values (TSV) rows using field-based comparison tests.

Input:
- Reads from files or standard input; multiple files are processed as one stream.
- Files ending in '.gz' are transparently decompressed.

Header behavior:
- --header / -H
  Treats the first non-empty line of the input as a header. The header is
  written once at the top of the output. Tests are applied only to data rows.

Tests and logic:
- Multiple tests can be specified. By default, all tests must pass (logical AND).
- Use --or to require that at least one test passes (logical OR).
- Use --invert to invert the overall match result (select non-matching rows).
- Use --count to print only the number of matching data rows.

Field syntax:
- All tests that take a <field-list> argument accept the same field list syntax
  as other tva commands: 1-based indices, ranges, header names, name ranges,
  and wildcards.
- Run `tva --help-fields` for a full description shared across tva commands.
"###,
        )
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
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva filter: {}", msg);
    std::process::exit(1);
}

enum NumericOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

enum TestKind {
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
    fn eval(&self, fields: &[&str]) -> bool {
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

struct PendingNumeric {
    spec: String,
    op: NumericOp,
}

struct PendingStrEq {
    spec: String,
    case_insensitive: bool,
    negated: bool,
}

struct PendingSubstr {
    spec: String,
    case_insensitive: bool,
    negated: bool,
}

struct PendingRegex {
    spec: String,
    case_insensitive: bool,
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
            v.push(PendingNumeric { spec, op: NumericOp::Gt });
        }
        for spec in args
            .get_many::<String>("ge")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric { spec, op: NumericOp::Ge });
        }
        for spec in args
            .get_many::<String>("lt")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric { spec, op: NumericOp::Lt });
        }
        for spec in args
            .get_many::<String>("le")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric { spec, op: NumericOp::Le });
        }
        for spec in args
            .get_many::<String>("eq")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric { spec, op: NumericOp::Eq });
        }
        for spec in args
            .get_many::<String>("ne")
            .map(|v| v.cloned().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            v.push(PendingNumeric { spec, op: NumericOp::Ne });
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

            if has_header && !header_seen && !line.is_empty() {
                header_seen = true;
                if !header_written && !count_only {
                    writer.write_fmt(format_args!("{}\n", line))?;
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
                header_struct =
                    Some(crate::libs::fields::Header::from_line(&line, delimiter));
                continue;
            }

            if line.is_empty() {
                continue;
            }

            let fields_vec: Vec<&str> = line.split(delimiter).collect();

            let header_ref = header_struct.as_ref();
            let mut tests: Vec<TestKind> = Vec::new();

            for spec in &empty_specs {
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    spec,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                tests.push(TestKind::Empty { fields: idxs });
            }
            for spec in &not_empty_specs {
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    spec,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                tests.push(TestKind::NotEmpty { fields: idxs });
            }
            for spec in &blank_specs {
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    spec,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                tests.push(TestKind::Blank { fields: idxs });
            }
            for spec in &not_blank_specs {
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    spec,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                tests.push(TestKind::NotBlank { fields: idxs });
            }

            for p in &numeric_specs {
                let (field_part, value_part) = split_spec(&p.spec);
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    &field_part,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                let value = value_part.parse::<f64>().unwrap_or_else(|_| {
                    arg_error(&format!("invalid numeric value `{}` in `{}`", value_part, p.spec))
                });
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

            for p in &str_eq_specs {
                let (field_part, value_part) = split_spec(&p.spec);
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    &field_part,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
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

            for p in &substr_specs {
                let (field_part, value_part) = split_spec(&p.spec);
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    &field_part,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                tests.push(TestKind::StrIn {
                    fields: idxs,
                    value: value_part.to_string(),
                    case_insensitive: p.case_insensitive,
                    negated: p.negated,
                });
            }

            for p in &regex_specs {
                let (field_part, pattern) = split_spec(&p.spec);
                let idxs = crate::libs::fields::parse_field_list_with_header(
                    &field_part,
                    header_ref,
                    delimiter,
                )
                .unwrap_or_else(|e| arg_error(&e));
                let regex = if p.case_insensitive {
                    Regex::new(&format!("(?i:{})", pattern))
                } else {
                    Regex::new(&pattern)
                }
                .unwrap_or_else(|e| {
                    arg_error(&format!("invalid regex `{}`: {}", pattern, e))
                });
                tests.push(TestKind::Regex { fields: idxs, regex });
            }

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

            if row_match {
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

fn split_spec(spec: &str) -> (String, String) {
    if let Some(pos) = spec.rfind(':') {
        let (left, right) = spec.split_at(pos);
        (left.to_string(), right[1..].to_string())
    } else {
        arg_error(&format!(
            "missing `:` separator in `{}` (expected <field-list>:<value>)",
            spec
        ));
    }
}

