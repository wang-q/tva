use clap::*;

pub fn make_subcommand() -> Command {
    Command::new("csv")
        .about("Converts TSV input to CSV")
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

    let mut tsv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(reader);

    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(writer);

    for result in tsv_reader.records() {
        let record = result?;
        csv_writer.write_record(&record)?;
    }

    csv_writer.flush()?;

    Ok(())
}
