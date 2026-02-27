use crate::libs::io::map_io_err;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::TsvRecord;
use clap::*;
use std::io::Write;

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
    let reader = crate::libs::io::reader(infile);
    let mut tsv_reader = TsvReader::with_capacity(reader, 512 * 1024);
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let mut data: Vec<TsvRecord> = Vec::new();
    let mut expected_fields: Option<usize> = None;
    let mut line_number: u64 = 0;

    tsv_reader.for_each_record(|line| {
        line_number += 1;

        let mut record = TsvRecord::new();
        record.parse_line(line, b'\t');
        let field_count = record.len();

        if let Some(exp) = expected_fields {
            if field_count != exp {
                eprintln!("line {} ({} fields):", line_number, field_count);
                eprintln!("  {}", String::from_utf8_lossy(line));
                eprintln!(
                    "tva transpose: structure check failed: line {} has {} fields (expected {})",
                    line_number, field_count, exp
                );
                std::process::exit(1);
            }
        } else {
            expected_fields = Some(field_count);
        }

        data.push(record);
        Ok(())
    }).map_err(map_io_err)?;

    if data.is_empty() {
        return Ok(());
    }

    let cols = expected_fields.unwrap_or(0);

    for c in 0..cols {
        for (r, row) in data.iter().enumerate() {
            if r > 0 {
                writer.write_all(b"\t")?;
            }
            if let Some(field) = row.get(c) {
                writer.write_all(field)?;
            }
        }
        writer.write_all(b"\n")?;
    }

    Ok(())
}
