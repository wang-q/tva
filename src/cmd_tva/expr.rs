use clap::builder::PossibleValue;
use clap::*;

use crate::libs::cli::{build_header_config, expr_common_args, get_delimiter};
use crate::libs::expr::runtime;
use crate::libs::expr::runtime::value::Value;
use crate::libs::expr::{fold_constants, parse_cached, resolve_columns};
use crate::libs::io::map_io_err;
use crate::libs::tsv::header::HeaderMode;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRow};
use ahash::{HashMap, HashMapExt};

/// Convert a Value to output string.
/// If the value is a List, it will be expanded to multiple columns (tab-separated).
/// Empty list returns "[]" to distinguish from no output.
/// Otherwise, returns the string representation of the value.
fn value_to_output(value: &Value) -> String {
    match value {
        Value::List(list) => {
            if list.is_empty() {
                // Empty list should output "[]"
                "[]".to_string()
            } else {
                // Expand list to multiple columns (tab-separated)
                list.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("\t")
            }
        }
        _ => value.to_string(),
    }
}

pub fn make_subcommand() -> Command {
    Command::new("expr")
        .about("Evaluates expressions for each row to create new row")
        .after_help(include_str!("../../docs/help/expr.md"))
        .args(expr_common_args())
        .arg(
            Arg::new("mode")
                .long("mode")
                .short('m')
                .num_args(1)
                .value_parser([
                    PossibleValue::new("eval").alias("e"),
                    PossibleValue::new("extend").alias("a"),
                    PossibleValue::new("mutate").alias("u"),
                    PossibleValue::new("skip-null").alias("s"),
                    PossibleValue::new("filter").alias("f"),
                ])
                .default_value("eval")
                .help(
                    "Output mode: eval (default), extend, mutate, skip-null, or filter",
                ),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mode = args
        .get_one::<String>("mode")
        .map(|s| s.as_str())
        .unwrap_or("eval");
    execute_with_mode(args, mode)
}

/// Execute with a specific mode override (used by mutate command).
pub fn execute_with_mode(args: &ArgMatches, mode: &str) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    // Get expression from -E or -F
    let expr_str = if let Some(expr) = args.get_one::<String>("expr") {
        expr.clone()
    } else if let Some(expr_file) = args.get_one::<String>("expr_file") {
        std::fs::read_to_string(expr_file)
            .map_err(|e| {
                anyhow::anyhow!("Failed to read expression file '{}': {}", expr_file, e)
            })?
            .trim()
            .to_string()
    } else {
        return Err(anyhow::anyhow!(
            "Either --expr/-E or --expr-file/-F must be provided"
        ));
    };

    if expr_str.is_empty() {
        return Err(anyhow::anyhow!("Expression cannot be empty"));
    }
    let skip_null = mode == "skip-null" || mode == "s";
    let filter_mode = mode == "filter" || mode == "f";
    let add_mode = mode == "extend" || mode == "a";
    let mutate_mode = mode == "mutate" || mode == "u";

    // Parse the expression with caching
    let mut parsed_expr = parse_cached(&expr_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression: {}", e))?;

    // For mutate mode, validate that expression has 'as @column' binding
    let mutate_target = if mutate_mode {
        let target = parsed_expr.get_mutate_target();
        if target.is_none() {
            return Err(anyhow::anyhow!(
                "mutate mode requires 'as @column' binding to specify which column to modify"
            ));
        }
        target
    } else {
        None
    };

    // Check if we have inline row data (debug mode)
    let row_values: Vec<String> = match args.get_many::<String>("row") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    // Get input files
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(), // Empty means no input files provided
    };

    // If we have inline row data, use debug mode (no input file needed)
    if !row_values.is_empty() {
        let headers_str = args.get_one::<String>("colnames");

        // Build headers if provided
        let headers: Option<Vec<String>> = headers_str
            .as_ref()
            .map(|h| h.split(',').map(|s| s.trim().to_string()).collect());

        // Create shared globals for cross-row persistence
        let globals = std::rc::Rc::new(std::cell::RefCell::new(HashMap::new()));

        // Process each row
        for (row_idx, row_str) in row_values.iter().enumerate() {
            let row: Vec<String> =
                row_str.split(',').map(|s| s.trim().to_string()).collect();
            let mut ctx = match headers.as_ref() {
                Some(h) => runtime::EvalContext::with_headers(&row, h),
                None => runtime::EvalContext::new(&row),
            };
            // Share globals across rows
            ctx.globals = globals.clone();
            // Set built-in global variables
            ctx.set_builtin_globals((row_idx + 1) as i64, "<inline>");

            let result = runtime::eval(&parsed_expr, &mut ctx)
                .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
            // Skip null results if --skip-null is enabled
            if skip_null && result.is_null() {
                continue;
            }
            // Filter mode: only output rows where expression evaluates to true
            if filter_mode && !result.as_bool() {
                continue;
            }
            // In filter mode, output the original row;
            // In add mode, output original row + expression result;
            // In mutate mode, modify the specified column and output the row;
            // Otherwise output the expression result
            if filter_mode {
                writeln!(writer, "{}", row.join("\t"))?;
            } else if add_mode {
                // Add mode: append expression result columns to original row
                let result_str = value_to_output(&result);
                if result_str.is_empty() {
                    writeln!(writer, "{}", row.join("\t"))?;
                } else {
                    writeln!(writer, "{}\t{}", row.join("\t"), result_str)?;
                }
            } else if mutate_mode {
                // Mutate mode: modify the specified column
                if let Some(ref target) = mutate_target {
                    let target_idx = if let Some(ref h) = headers {
                        // Find column index by name
                        h.iter().position(|col| col == target)
                    } else {
                        // No headers, try to parse as 1-based index
                        target.parse::<usize>().ok().map(|i| i - 1)
                    };

                    if let Some(idx) = target_idx {
                        let mut new_row = row.clone();
                        if idx < new_row.len() {
                            new_row[idx] = value_to_output(&result);
                        }
                        writeln!(writer, "{}", new_row.join("\t"))?;
                    } else {
                        return Err(anyhow::anyhow!(
                            "mutate target column '{}' not found",
                            target
                        ));
                    }
                }
            } else {
                writeln!(writer, "{}", value_to_output(&result))?;
            }
        }

        return Ok(());
    }

    // If no input files and no row data, evaluate expression with empty context
    if infiles.is_empty() {
        let fields: Vec<String> = Vec::new();
        let mut ctx = runtime::EvalContext::new(&fields);
        let result = runtime::eval(&parsed_expr, &mut ctx)
            .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;
        // Skip null results if --skip-null is enabled
        if skip_null && result.is_null() {
            return Ok(());
        }
        // Filter mode with no input: only output if expression evaluates to true
        if filter_mode {
            if result.as_bool() {
                // In filter mode with no input, output empty line (original row is empty)
                writeln!(writer)?;
            }
            return Ok(());
        }
        writeln!(writer, "{}", value_to_output(&result))?;
        return Ok(());
    }

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, false).map_err(|e| anyhow::anyhow!(e))?;
    let has_header = header_config.enabled;

    let opt_delimiter = get_delimiter(args, "delimiter")?;
    let delim_byte = opt_delimiter;

    let mut header_written = false;
    let mut headers: Vec<String> = Vec::new();

    // Create shared globals for cross-row persistence across all files
    let globals = std::rc::Rc::new(std::cell::RefCell::new(HashMap::new()));
    // Use Cell for interior mutability in the closure
    let row_num = std::cell::Cell::new(1i64);

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let filename = input.name.clone();
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
                        .split(|c| c == opt_delimiter as char)
                        .map(|s| s.to_string())
                        .collect();

                    // Write header (before resolve_columns to preserve column names)
                    if filter_mode || mutate_mode {
                        // In filter and mutate mode, preserve original header
                        writeln!(writer, "{}", headers.join("\t"))?;
                    } else if add_mode {
                        // Add mode: append expression header names to original headers
                        let header_names = parsed_expr.header_names(&headers);
                        if header_names.is_empty() {
                            writeln!(writer, "{}", headers.join("\t"))?;
                        } else {
                            writeln!(
                                writer,
                                "{}\t{}",
                                headers.join("\t"),
                                header_names.join("\t")
                            )?;
                        }
                    } else {
                        // Generate header names using the new header_names() method
                        // This handles as @name, @column_name, @1 with headers, etc.
                        // For list expressions like [@a, @b], returns ["a", "b"]
                        let header_names = parsed_expr.header_names(&headers);
                        writeln!(writer, "{}", header_names.join("\t"))?;
                    }
                    header_written = true;

                    // Optimize expression: resolve column names to indices
                    resolve_columns(&mut parsed_expr, &headers).map_err(|e| {
                        anyhow::anyhow!("Failed to resolve columns: {}", e)
                    })?;
                    // Fold constant expressions for better performance
                    fold_constants(&mut parsed_expr);
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
        let filter_mode_flag = filter_mode;
        let add_mode_flag = add_mode;
        let mutate_mode_flag = mutate_mode;
        let mutate_target_ref = mutate_target.as_ref();
        let globals_clone = globals.clone();
        let result: std::io::Result<()> =
            tsv_reader.for_each_row(delim_byte, |row: &TsvRow| {
                // Extract fields from TsvRow using ends array
                let fields: Vec<String> = (1..=row.ends.len())
                    .map(|idx| {
                        row.get_bytes(idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    })
                    .collect();

                // Evaluate expression
                let mut ctx = if headers.is_empty() {
                    runtime::EvalContext::new(&fields)
                } else {
                    runtime::EvalContext::with_headers(&fields, &headers)
                };
                // Share globals across rows
                ctx.globals = globals_clone.clone();
                // Set built-in global variables
                let current_row = row_num.get();
                ctx.set_builtin_globals(current_row, &filename);
                row_num.set(current_row + 1);

                let result = runtime::eval(&parsed_expr, &mut ctx)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                // Skip null results if --skip-null is enabled
                if skip_null_flag && result.is_null() {
                    return Ok(());
                }

                // Filter mode: only output rows where expression evaluates to true
                if filter_mode_flag {
                    if result.as_bool() {
                        // Output original row
                        writeln!(writer, "{}", fields.join("\t"))
                            .map_err(|e| std::io::Error::other(e.to_string()))?;
                    }
                    return Ok(());
                }

                // Add mode: append expression result to original row
                if add_mode_flag {
                    let result_str = value_to_output(&result);
                    if result_str.is_empty() {
                        writeln!(writer, "{}", fields.join("\t"))
                            .map_err(|e| std::io::Error::other(e.to_string()))?;
                    } else {
                        writeln!(writer, "{}\t{}", fields.join("\t"), result_str)
                            .map_err(|e| std::io::Error::other(e.to_string()))?;
                    }
                    return Ok(());
                }

                // Mutate mode: modify the specified column
                if mutate_mode_flag {
                    if let Some(target) = mutate_target_ref {
                        let target_idx = headers.iter().position(|col| col == target);
                        if let Some(idx) = target_idx {
                            let mut new_row = fields.clone();
                            if idx < new_row.len() {
                                new_row[idx] = value_to_output(&result);
                            }
                            writeln!(writer, "{}", new_row.join("\t"))
                                .map_err(|e| std::io::Error::other(e.to_string()))?;
                        } else {
                            return Err(std::io::Error::other(format!(
                                "mutate target column '{}' not found",
                                target
                            )));
                        }
                    }
                    return Ok(());
                }

                // Output result
                writeln!(writer, "{}", value_to_output(&result))
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                Ok(())
            });

        result.map_err(|e| anyhow::anyhow!("Error processing file: {}", e))?;
    }

    Ok(())
}
