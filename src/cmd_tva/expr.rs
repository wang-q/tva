use clap::*;

use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::expr::runtime;
use crate::libs::expr::{fold_constants, parse_cached, resolve_columns};
use crate::libs::io::map_io_err;
use crate::libs::tsv::header::HeaderMode;
use crate::libs::tsv::reader::TsvReader;

pub fn make_subcommand() -> Command {
    Command::new("expr")
        .about("Evaluates expression for each row to create new row")
        .after_help(include_str!("../../docs/help/expr.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("expr")
                .long("expr")
                .short('E')
                .num_args(1)
                .required(true)
                .help("Expression to evaluate (e.g., '@price * @qty as @total')"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter character (default: TAB)"),
        )
        .arg(
            Arg::new("colnames")
                .long("colnames")
                .short('n')
                .num_args(1)
                .help("Comma-separated column names for reference (e.g., 'name,age')"),
        )
        .arg(
            Arg::new("row")
                .long("row")
                .short('r')
                .action(ArgAction::Append)
                .help("Comma-separated row values to evaluate against (e.g., 'Alice,30'). Can be specified multiple times for multiple rows."),
        )
        .arg(
            Arg::new("skip_null")
                .long("skip-null")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Skip rows where the expression result is null"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let expr_str = args.get_one::<String>("expr").unwrap();
    let skip_null = args.get_flag("skip_null");

    // Parse the expression with caching
    let mut parsed_expr = parse_cached(expr_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression: {}", e))?;

    // Check if we have inline row data (debug mode)
    let row_values: Vec<String> = match args.get_many::<String>("row") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    // Get input files
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    // If we have inline row data, use debug mode (no input file needed)
    if !row_values.is_empty() {
        let headers_str = args.get_one::<String>("colnames");

        // Build headers if provided
        let headers: Option<Vec<String>> = headers_str
            .as_ref()
            .map(|h| h.split(',').map(|s| s.trim().to_string()).collect());

        // Process each row
        for row_str in &row_values {
            let row: Vec<String> =
                row_str.split(',').map(|s| s.trim().to_string()).collect();
            let mut ctx = match headers.as_ref() {
                Some(h) => runtime::EvalContext::with_headers(&row, h),
                None => runtime::EvalContext::new(&row),
            };
            let result = runtime::eval(&parsed_expr, &mut ctx)
                .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
            // Skip null results if --skip-null is enabled
            if skip_null && result.is_null() {
                continue;
            }
            writeln!(writer, "{}", result.to_string())?;
        }

        return Ok(());
    }

    // If no input files and no row data, evaluate expression with empty context
    if infiles.is_empty() {
        let row: Vec<String> = Vec::new();
        let mut ctx = runtime::EvalContext::new(&row);
        let result = runtime::eval(&parsed_expr, &mut ctx)
            .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
        // Skip null results if --skip-null is enabled
        if !skip_null || !result.is_null() {
            writeln!(writer, "{}", result.to_string())?;
        }
        return Ok(());
    }

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, false).map_err(|e| anyhow::anyhow!(e))?;
    let has_header = header_config.enabled;

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .cloned()
        .unwrap_or_else(|| "\t".to_string());
    let mut chars = delimiter_str.chars();
    let delimiter = chars.next().unwrap_or('\t');
    if chars.next().is_some() {
        return Err(anyhow::anyhow!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        ));
    }
    let delim_byte = delimiter as u8;

    let mut header_written = false;
    let mut headers: Vec<String> = Vec::new();

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut tsv_reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        if has_header {
            if !header_written {
                // First file: read header
                let header_result = tsv_reader
                    .read_header_mode(header_config.mode)
                    .map_err(map_io_err)?;

                if let Some(info) = header_result {
                    // Parse column names from column_names_line or first line
                    let header_line = info
                        .column_names_line
                        .as_ref()
                        .or(info.lines.first())
                        .cloned()
                        .unwrap_or_default();
                    headers = String::from_utf8_lossy(&header_line)
                        .split(|c| c == delimiter)
                        .map(|s| s.to_string())
                        .collect();

                    // Save original header name before optimization
                    let header_name = parsed_expr.last_expr().format();

                    // Optimize expression: resolve column names to indices
                    resolve_columns(&mut parsed_expr, &headers);
                    // Fold constant expressions for better performance
                    fold_constants(&mut parsed_expr);

                    // Write header from the original formatted representation
                    writeln!(writer, "{}", header_name)?;
                    header_written = true;
                }
            } else {
                // Subsequent files: skip header
                let _ = tsv_reader
                    .read_header_mode(HeaderMode::FirstLine)
                    .map_err(map_io_err)?;
            }
        }

        // Process data rows
        let skip_null_flag = skip_null;
        let result: std::io::Result<()> = tsv_reader.for_each_record(|record| {
            // Split record into fields
            let row: Vec<String> = record
                .split(|&b| b == delim_byte)
                .map(|s| String::from_utf8_lossy(s).to_string())
                .collect();

            // Evaluate expression
            let mut ctx = if headers.is_empty() {
                runtime::EvalContext::new(&row)
            } else {
                runtime::EvalContext::with_headers(&row, &headers)
            };
            let result = runtime::eval(&parsed_expr, &mut ctx).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;

            // Skip null results if --skip-null is enabled
            if skip_null_flag && result.is_null() {
                return Ok(());
            }

            // Output result
            writeln!(writer, "{}", result.to_string()).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;

            Ok(())
        });

        result.map_err(|e| anyhow::anyhow!("Error processing file: {}", e))?;
    }

    Ok(())
}
