use clap::*;
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("transpose")
        .about("Transposes TSV table in strict mode")
        .after_help(include_str!("../../docs/help/transpose.md"))
        .arg(
            Arg::new("infile")
                .num_args(0..=1)
                .default_value("stdin")
                .index(1)
                .help("Input TSV file to transpose (default: stdin)"),
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

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let mut reader = crate::libs::io::reader(infile);
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let mut data: Vec<Vec<String>> = Vec::new();
    let mut expected_fields: Option<usize> = None;
    let mut line_number: u64 = 0;

    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        line_number += 1;

        let fields: Vec<String> = if line.is_empty() {
            Vec::new()
        } else {
            line.split('\t').map(|s| s.to_string()).collect()
        };

        let field_count = fields.len();

        if let Some(exp) = expected_fields {
            if field_count != exp {
                eprintln!("line {} ({} fields):", line_number, field_count);
                eprintln!("  {}", line);
                eprintln!(
                    "tva transpose: structure check failed: line {} has {} fields (expected {})",
                    line_number, field_count, exp
                );
                std::process::exit(1);
            }
        } else {
            expected_fields = Some(field_count);
        }

        data.push(fields);
    }

    if data.is_empty() {
        return Ok(());
    }

    let cols = expected_fields.unwrap_or(0);
    let rows = data.len();

    for c in 0..cols {
        for r in 0..rows {
            if r > 0 {
                writer.write_all(b"\t")?;
            }
            writer.write_all(data[r][c].as_bytes())?;
        }
        writer.write_all(b"\n")?;
    }

    Ok(())
}
