use calamine::{open_workbook_auto, Data, Reader};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("xlsx")
        .about("Converts XLSX/XLS input to TSV")
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input XLSX/XLS file to process"),
        )
        .arg(
            Arg::new("sheet")
                .long("sheet")
                .short('s')
                .num_args(1)
                .help("Sheet name to process (default: first sheet)"),
        )
        .arg(
            Arg::new("list-sheets")
                .long("list-sheets")
                .action(ArgAction::SetTrue)
                .help("List all sheet names and exit"),
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
    let mut workbook = open_workbook_auto(infile)
        .map_err(|e| anyhow::anyhow!("Failed to open file: {}", e))?;

    if args.get_flag("list-sheets") {
        for sheet in workbook.sheet_names() {
            println!("{}", sheet);
        }
        return Ok(());
    }

    let sheet_name = if let Some(name) = args.get_one::<String>("sheet") {
        name.clone()
    } else {
        workbook
            .sheet_names()
            .first()
            .ok_or_else(|| anyhow::anyhow!("No sheets found"))?
            .clone()
    };

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| anyhow::anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;

    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    for row in range.rows() {
        let mut first = true;
        for cell in row {
            if !first {
                writer.write_all(b"\t")?;
            }

            let val = match cell {
                Data::Empty => String::new(),
                Data::String(s) => {
                    s.replace('\t', " ").replace('\n', " ").replace('\r', " ")
                }
                Data::Float(f) => f.to_string(),
                Data::Int(i) => i.to_string(),
                Data::Bool(b) => b.to_string(),
                Data::Error(e) => format!("{:?}", e),
                Data::DateTime(f) => f.to_string(),
                _ => String::new(), // Handle other variants if any
            };

            writer.write_all(val.as_bytes())?;
            first = false;
        }
        writer.write_all(b"\n")?;
    }

    writer.flush()?;

    Ok(())
}
