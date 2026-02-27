use clap::*;
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("md")
        .about("Converts TSV file to markdown table")
        .after_help(include_str!("../../docs/help/md.md"))
        .arg(
            Arg::new("infile")
                .num_args(0..=1)
                .default_value("stdin")
                .index(1)
                .help("Input TSV file to process (default: stdin)"),
        )
        .arg(
            Arg::new("center")
                .long("center")
                .short('c')
                .num_args(1)
                .help("Center-aligned columns"),
        )
        .arg(
            Arg::new("right")
                .long("right")
                .short('r')
                .num_args(1)
                .help("Right-aligned columns"),
        )
        .arg(
            Arg::new("num")
                .long("num")
                .action(ArgAction::SetTrue)
                .help("Right-aligning numeric columns"),
        )
        .arg(
            Arg::new("fmt")
                .long("fmt")
                .action(ArgAction::SetTrue)
                .help("Format numeric columns and enable the `--num` option"),
        )
        .arg(
            Arg::new("digits")
                .long("digits")
                .num_args(1)
                .default_value("0")
                .value_parser(value_parser!(usize))
                .help("Decimal digits"),
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
    let mut reader = crate::libs::tsv::reader::TsvReader::new(
        crate::libs::io::raw_reader(args.get_one::<String>("infile").unwrap())
    );

    let mut opt_center: intspan::IntSpan = if args.contains_id("center") {
        crate::libs::tsv::fields::fields_to_ints(
            args.get_one::<String>("center").unwrap(),
        )
    } else {
        intspan::IntSpan::new()
    };
    let mut opt_right: intspan::IntSpan = if args.contains_id("right") {
        crate::libs::tsv::fields::fields_to_ints(
            args.get_one::<String>("right").unwrap(),
        )
    } else {
        intspan::IntSpan::new()
    };
    let mut is_num = args.get_flag("num");
    let is_fmt = args.get_flag("fmt");
    if is_fmt {
        is_num = true;
    }
    let opt_digits: usize = *args.get_one("digits").unwrap();

    //----------------------------
    // Output
    //----------------------------
    let mut is_numeric_column = vec![];

    // Read all data into memory (required for alignment detection and markdown table structure)
    // TsvReader iterates over bytes, we convert to String for now as markdown formatting 
    // and numeric parsing needs strings.
    // Optimization: could store as Vec<Vec<Vec<u8>>> or similar but markdown formatter likely needs strings.
    let mut data: Vec<Vec<String>> = Vec::new();
    
    reader.for_each_record(|line| {
        let fields: Vec<String> = line
            .split(|&b| b == b'\t')
            .map(|field| String::from_utf8_lossy(field).to_string())
            .collect();
        data.push(fields);
        Ok(())
    })?;

    let mut table = String::new();
    if !data.is_empty() {
        let num_columns = data[0].len();
        if is_num {
            // Determine if each column is numeric
            is_numeric_column = vec![true; num_columns];

            for row in data.iter().skip(1) {
                // Skip the header row
                for (i, value) in row.iter().enumerate() {
                    if i < num_columns && is_numeric_column[i] && value.parse::<f64>().is_err() {
                        is_numeric_column[i] = false;
                    }
                }
            }

            for (i, &is_numeric) in
                is_numeric_column.iter().enumerate().take(num_columns)
            {
                if is_numeric {
                    opt_center.remove_n((i + 1) as i32);
                    opt_right.add_n((i + 1) as i32);
                }
            }
        } else if is_fmt {
             // If fmt is on but num check logic didn't run (though we set is_num=true if is_fmt is true),
             // actually logic above handles it. But let's ensure is_numeric_column is sized if needed later.
             if is_numeric_column.is_empty() {
                 is_numeric_column = vec![false; num_columns];
             }
        } else {
             if is_numeric_column.is_empty() {
                 is_numeric_column = vec![false; num_columns];
             }
        }

        // Print the Markdown table
        for (i, row) in data.iter().enumerate() {
            let formatted_row: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(j, value)| {
                    // Don't touch first row
                    if i == 0 {
                        value.to_string()
                    } else if is_fmt && j < is_numeric_column.len() && is_numeric_column[j] {
                        if let Ok(num) = value.parse::<f64>() {
                            crate::libs::number::format_number(num, opt_digits)
                        } else {
                            value.to_string()
                        }
                    } else {
                        value.to_string()
                    }
                })
                .collect();
            table += format!("| {} |\n", formatted_row.join(" | ")).as_str();

            // Print the header separator
            if i == 0 {
                let separator: Vec<String> = (0..num_columns)
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|&j| {
                        if opt_right.contains((j + 1) as i32) {
                            "---:".to_string()
                        } else if opt_center.contains((j + 1) as i32) {
                            ":---:".to_string()
                        } else {
                            "---".to_string()
                        }
                    })
                    .collect();
                table += format!("| {} |\n", separator.join(" | ")).as_str();
            }
        }
    }

    if !table.is_empty() {
        writer.write_fmt(format_args!(
            "{}",
            markdown_table_formatter::format_tables(table)
        ))?;
    }

    Ok(())
}
