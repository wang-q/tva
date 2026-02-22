use clap::*;
use std::collections::HashSet;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("dedup")
        .about("Deduplicates TSV rows from one or more files")
        .after_help(
            r###"
Description:
Deduplicates rows of one or more tab-separated values (TSV) files without sorting.

Notes:
* Supports plain text and gzipped (.gz) TSV files
* Reads from stdin if no input file is given or if input file is 'stdin'
* Keeps a 64-bit hash for each unique key; ~8 bytes of memory per unique row
* Only the first occurrence of each key is kept; occurrences are not counted

Examples:
1. Deduplicate whole rows
   tva dedup tests/genome/ctg.tsv

2. Deduplicate by column 2
   tva dedup tests/genome/ctg.tsv -f 2
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

    let opt_fields: intspan::IntSpan = if args.contains_id("fields") {
        crate::libs::fields::fields_to_ints(args.get_one::<String>("fields").unwrap())
    } else {
        intspan::IntSpan::new()
    };

    let mut subject_set: HashSet<u64> = HashSet::new();

    for infile in &infiles {
        let reader = crate::libs::io::reader(infile);

        for line in reader.lines().map_while(Result::ok) {
            let subject = if opt_fields.is_empty() {
                // whole line
                xxhash_rust::xxh3::xxh3_64(line.as_bytes())
            } else {
                // Get elements at specified indices
                let fields: Vec<&str> = line.split('\t').collect();
                let subset: Vec<&str> = opt_fields
                    .elements()
                    .iter()
                    .filter_map(|&i| fields.get(i as usize - 1))
                    .copied()
                    .collect();
                let concat = subset.join("\t");
                xxhash_rust::xxh3::xxh3_64(&concat.into_bytes())
            };

            if !subject_set.contains(&subject) {
                writer.write_fmt(format_args!("{}\n", line))?;
                subject_set.insert(subject);
            }
        }
    }

    Ok(())
}
