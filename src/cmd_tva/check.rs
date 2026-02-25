use clap::*;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("check")
        .about("Checks TSV table structure for consistent field counts")
        .after_help(include_str!("../../docs/help/check.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to check (default: stdin)"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let mut total_lines: u64 = 0;
    let mut expected_fields: Option<usize> = None;

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(err) => {
                    eprintln!("tva check: error reading input: {}", err);
                    std::process::exit(1);
                }
            };

            total_lines += 1;
            let fields = if line.is_empty() {
                0
            } else {
                line.split('\t').count()
            };

            if let Some(exp) = expected_fields {
                if fields != exp {
                    eprintln!("line {} ({} fields):", total_lines, fields);
                    eprintln!("  {}", line);
                    eprintln!(
                        "tva check: structure check failed: line {} has {} fields (expected {})",
                        total_lines, fields, exp
                    );
                    std::process::exit(1);
                }
            } else {
                expected_fields = Some(fields);
            }
        }
    }

    match expected_fields {
        Some(fields) => {
            println!("{} lines, {} fields", total_lines, fields);
        }
        None => {
            println!("0 lines, 0 fields");
        }
    }

    Ok(())
}
