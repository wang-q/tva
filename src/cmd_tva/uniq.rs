use clap::*;
use rapidhash::rapidhash;
use std::collections::HashSet;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("uniq")
        .about("Deduplicates TSV rows from one or more files")
        .after_help(
            r###"
Deduplicates rows of one or more tab-separated values (TSV) files without sorting.

Notes:
* Supports plain text and gzipped (.gz) TSV files
* Reads from stdin if no input file is given or if input file is 'stdin'
* Keeps a 64-bit hash for each unique key; ~8 bytes of memory per unique row
* Only the first occurrence of each key is kept; occurrences are not counted

Examples:
1. Deduplicate whole rows
   tva uniq tests/genome/ctg.tsv

2. Deduplicate by column 2
   tva uniq tests/genome/ctg.tsv -f 2
"###,
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("fields")
                .long("fields")
                .short('f')
                .num_args(1)
                .help("TSV fields (1-based) to use as dedup key"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each input as a header; only the first header is output"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let fields_spec: Option<String> = args.get_one::<String>("fields").cloned();

    let has_header = args.get_flag("header");
    let mut header_written = false;
    let mut header: Option<crate::libs::fields::Header> = None;
    let mut key_fields: Option<Vec<usize>> = None;

    let mut subject_set: HashSet<u64> = HashSet::new();

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        let mut is_first_line = true;

        for line in reader.lines().map_while(Result::ok) {
            if has_header && is_first_line {
                if header.is_none() {
                    header = Some(crate::libs::fields::Header::from_line(&line, '\t'));
                    if let Some(ref spec) = fields_spec {
                        key_fields = Some(
                            crate::libs::fields::parse_field_list_with_header(
                                spec,
                                header.as_ref(),
                                '\t',
                            )
                            .map_err(|e| anyhow::anyhow!(e))?,
                        );
                    }
                }

                if !header_written {
                    writer.write_fmt(format_args!("{}\n", line))?;
                    header_written = true;
                }
                is_first_line = false;
                continue;
            }

            is_first_line = false;

            if key_fields.is_none() {
                if let Some(ref spec) = fields_spec {
                    key_fields = Some(
                        crate::libs::fields::parse_field_list_with_header(
                            spec,
                            None,
                            '\t',
                        )
                        .map_err(|e| anyhow::anyhow!(e))?,
                    );
                }
            }

            let subject = if key_fields.as_ref().map_or(true, |v| v.is_empty()) {
                rapidhash(line.as_bytes())
            } else {
                let fields: Vec<&str> = line.split('\t').collect();
                let subset: Vec<&str> = key_fields
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|&i| fields.get(i - 1))
                    .copied()
                    .collect();
                let concat = subset.join("\t");
                rapidhash(concat.as_bytes())
            };

            if !subject_set.contains(&subject) {
                writer.write_fmt(format_args!("{}\n", line))?;
                subject_set.insert(subject);
            }
        }
    }

    Ok(())
}

