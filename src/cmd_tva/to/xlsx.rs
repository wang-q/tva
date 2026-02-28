use crate::libs::io::map_io_err;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::Row;
use clap::{Arg, ArgAction, ArgMatches, Command};
use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatText, Format, FormatAlign,
    FormatBorder, Workbook,
};
use std::path::Path;

pub fn make_subcommand() -> Command {
    Command::new("xlsx")
        .about("Converts TSV input to Excel (xlsx)")
        .after_help(include_str!("../../../docs/help/to_xlsx.md"))
        .arg(
            Arg::new("infile")
                .required(true)
                .index(1)
                .help("Input TSV file to process"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .help("Output filename. [stdout] for screen (not supported for xlsx)"),
        )
        .arg(
            Arg::new("sheet")
                .long("sheet")
                .short('s')
                .num_args(1)
                .help("Sheet name (default: infile basename)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat first non-empty row as header and style it"),
        )
        .arg(
            Arg::new("font-name")
                .long("font-name")
                .num_args(1)
                .default_value("Arial")
                .help("Font name"),
        )
        .arg(
            Arg::new("font-size")
                .long("font-size")
                .num_args(1)
                .default_value("10")
                .help("Font size"),
        )
        .arg(
            Arg::new("le")
                .long("le")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD <= NUM"),
        )
        .arg(
            Arg::new("ge")
                .long("ge")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: FIELD >= NUM"),
        )
        .arg(
            Arg::new("bt")
                .long("bt")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Numeric comparison: MIN <= FIELD <= MAX"),
        )
        .arg(
            Arg::new("str-in-fld")
                .long("str-in-fld")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Substring test: FIELD contains STR"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let outfile = args.get_one::<String>("outfile");

    // Determine output filename
    let outfile_path = if let Some(out) = outfile {
        out.clone()
    } else {
        let path = Path::new(infile);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        format!("{}.xlsx", stem)
    };

    if outfile_path == "stdout" {
        return Err(anyhow::anyhow!(
            "Output to stdout is not supported for XLSX format"
        ));
    }

    // Determine sheet name
    let sheet_name = if let Some(sheet) = args.get_one::<String>("sheet") {
        sheet.clone()
    } else {
        let path = Path::new(infile);
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    };

    // Font settings
    let font_name = args.get_one::<String>("font-name").unwrap();
    let font_size = args
        .get_one::<String>("font-size")
        .unwrap()
        .parse::<f64>()?;

    // Create workbook and worksheet
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name(&sheet_name)?;

    // Formats
    let header_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x99CCFF)) // #99CCFF
        .set_bold()
        .set_border_bottom(FormatBorder::Medium)
        .set_font_name(font_name)
        .set_font_size(font_size);

    let normal_format = Format::new()
        .set_font_color(Color::Black)
        .set_font_name(font_name)
        .set_font_size(font_size);

    let _highlight_format = Format::new()
        .set_font_color(Color::Blue)
        .set_bold()
        .set_font_name(font_name)
        .set_font_size(font_size);

    let le_format = Format::new()
        .set_background_color(Color::RGB(0xFFC7CE)) // #FFC7CE
        .set_font_color(Color::RGB(0x9C0006)) // #9C0006
        .set_font_name(font_name)
        .set_font_size(font_size);

    let ge_format = Format::new()
        .set_background_color(Color::RGB(0xFFEB9C)) // #FFEB9C
        .set_font_color(Color::RGB(0x9C6500)) // #9C6500
        .set_font_name(font_name)
        .set_font_size(font_size);

    let bt_format = Format::new()
        .set_background_color(Color::RGB(0xC6EFCE)) // #C6EFCE
        .set_font_color(Color::RGB(0x006100)) // #006100
        .set_font_name(font_name)
        .set_font_size(font_size);

    let contain_format = Format::new()
        .set_font_color(Color::Blue)
        .set_bold()
        .set_font_name(font_name)
        .set_font_size(font_size);

    // Read TSV and write to XLSX
    let reader = crate::libs::io::reader(infile);
    let mut tsv_reader = TsvReader::with_capacity(reader, 512 * 1024);
    let mut row_cursor: u32 = 0;
    let has_header = args.get_flag("header");

    tsv_reader.for_each_row(|row| {
        let format = if row_cursor == 0 && has_header {
            &header_format
        } else {
            &normal_format
        };

        // Iterate fields using Row trait
        let mut i = 1;
        while let Some(field_bytes) = row.get_bytes(i) {
            let col_idx = (i - 1) as u16;
            
            // Try to parse as number if possible, otherwise write as string
            // Avoid string allocation if possible for checking
            if let Ok(s) = std::str::from_utf8(field_bytes) {
                if let Ok(val) = s.parse::<f64>() {
                    worksheet.write_number_with_format(
                        row_cursor,
                        col_idx,
                        val,
                        format,
                    ).map_err(map_io_err)?;
                } else {
                    worksheet.write_string_with_format(
                        row_cursor,
                        col_idx,
                        s,
                        format,
                    ).map_err(map_io_err)?;
                }
            } else {
                // Fallback for non-utf8 (shouldn't happen in valid TSV usually)
                worksheet.write_string_with_format(
                    row_cursor,
                    col_idx,
                    String::from_utf8_lossy(field_bytes).as_ref(),
                    format,
                ).map_err(map_io_err)?;
            }
            i += 1;
        }
        row_cursor += 1;
        Ok(())
    }).map_err(map_io_err)?;

    // Freeze header if present
    if has_header {
        worksheet.set_freeze_panes(1, 0)?;
    }

    // Apply conditional formatting
    let start_row = if has_header { 1 } else { 0 };
    let end_row = if row_cursor > 0 { row_cursor - 1 } else { 0 };

    if start_row <= end_row {
        if let Some(values) = args.get_many::<String>("le") {
            for val in values {
                if let Some((col_str, limit_str)) = val.split_once(':') {
                    let col = col_str.parse::<u16>()? - 1; // 1-based index
                    let limit = limit_str.parse::<f64>()?;

                    let condition = ConditionalFormatCell::new()
                        .set_rule(
                            rust_xlsxwriter::ConditionalFormatCellRule::LessThanOrEqualTo(limit),
                        )
                        .set_format(le_format.clone());

                    worksheet.add_conditional_format(
                        start_row, col, end_row, col, &condition,
                    )?;
                }
            }
        }

        if let Some(values) = args.get_many::<String>("ge") {
            for val in values {
                if let Some((col_str, limit_str)) = val.split_once(':') {
                    let col = col_str.parse::<u16>()? - 1;
                    let limit = limit_str.parse::<f64>()?;

                    let condition = ConditionalFormatCell::new()
                        .set_rule(
                            rust_xlsxwriter::ConditionalFormatCellRule::GreaterThanOrEqualTo(limit),
                        )
                        .set_format(ge_format.clone());

                    worksheet.add_conditional_format(
                        start_row, col, end_row, col, &condition,
                    )?;
                }
            }
        }

        if let Some(values) = args.get_many::<String>("bt") {
            for val in values {
                let parts: Vec<&str> = val.split(':').collect();
                if parts.len() == 3 {
                    let col = parts[0].parse::<u16>()? - 1;
                    let min = parts[1].parse::<f64>()?;
                    let max = parts[2].parse::<f64>()?;

                    let condition = ConditionalFormatCell::new()
                        .set_rule(rust_xlsxwriter::ConditionalFormatCellRule::Between(
                            min, max,
                        ))
                        .set_format(bt_format.clone());

                    worksheet.add_conditional_format(
                        start_row, col, end_row, col, &condition,
                    )?;
                }
            }
        }

        if let Some(values) = args.get_many::<String>("str-in-fld") {
            for val in values {
                if let Some((col_str, text)) = val.split_once(':') {
                    let col = col_str.parse::<u16>()? - 1;

                    let condition = ConditionalFormatText::new()
                        .set_rule(rust_xlsxwriter::ConditionalFormatTextRule::Contains(
                            text.to_string(),
                        ))
                        .set_format(contain_format.clone());

                    worksheet.add_conditional_format(
                        start_row, col, end_row, col, &condition,
                    )?;
                }
            }
        }
    }

    // Autofit the columns.
    worksheet.autofit();

    workbook.save(outfile_path)?;

    Ok(())
}
