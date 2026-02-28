use crate::libs::io::map_io_err;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::Row;
use clap::*;

pub fn make_subcommand() -> Command {
    Command::new("csv")
        .about("Converts TSV input to CSV")
        .after_help(include_str!("../../../docs/help/to_csv.md"))
        .arg(
            Arg::new("infile")
                .num_args(0..=1)
                .default_value("stdin")
                .index(1)
                .help("Input TSV file to process (default: stdin)"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value(",")
                .help("CSV field delimiter character (default: ,)"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let reader = crate::libs::io::reader(infile);
    let writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or(",");

    let delimiter = if delimiter_str.len() == 1 {
        delimiter_str.as_bytes()[0]
    } else {
        anyhow::bail!("delimiter must be a single byte");
    };

    let mut tsv_reader = TsvReader::with_capacity(reader, 512 * 1024);

    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(writer);

    tsv_reader.for_each_row(|row| {
        // Iterate through fields (1-based index in Row trait)
        // Row trait has no iterator, but we can access `row.ends` in TsvRow.
        // Or simply implement an iterator for TsvRow or access underlying slice via get_bytes
        // TsvRow has ends, so we can iterate indices.
        // Wait, TsvRow::ends length is the number of fields.
        
        let mut fields = Vec::with_capacity(row.ends.len());
        let mut i = 1;
        while let Some(field) = row.get_bytes(i) {
            fields.push(field);
            i += 1;
        }
        csv_writer.write_record(fields)?;
        Ok(())
    }).map_err(map_io_err)?;

    csv_writer.flush()?;

    Ok(())
}
