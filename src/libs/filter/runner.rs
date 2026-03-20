use super::builder::build_tests;
use super::config::FilterConfig;
use super::engine::TestKind;
use crate::libs::io::map_io_err;
use crate::libs::tsv::record::TsvRow;
use anyhow::Result;
use std::io::Write;

pub fn run_filter<W: Write>(
    infiles: &[String],
    writer: &mut W,
    config: FilterConfig,
) -> Result<()> {
    let mut total_matched: u64 = 0;
    let mut header_written = false;
    let mut delim_buf = [0u8; 4];
    let delim_bytes = config.delimiter.encode_utf8(&mut delim_buf).as_bytes();
    let delim_byte = config.delimiter as u8;

    let has_header = config.header_config.enabled;

    let tests_without_header: Option<Vec<TestKind>> = if has_header {
        None
    } else {
        Some(
            build_tests(None, config.delimiter, config.as_spec_config())
                .map_err(|e| anyhow::anyhow!(e))?,
        )
    };

    for input in crate::libs::io::raw_input_sources(infiles)? {
        let mut tsv_reader = crate::libs::tsv::reader::TsvReader::new(input.reader);
        let mut tests_with_header: Option<Vec<TestKind>> = None;

        if has_header {
            // Use read_header_mode to support all header modes
            let header_result = tsv_reader
                .read_header_mode(config.header_config.mode)
                .map_err(map_io_err)?;

            if let Some(header_info) = header_result {
                // Get the column names line for field name resolution
                if let Some(column_names_bytes) = header_info.column_names_line {
                    if !header_written && !config.count_only {
                        // Write the column names line (not all header lines for simplicity)
                        if let Some(ref lbl) = config.label_header {
                            writer.write_all(&column_names_bytes)?;
                            writer.write_all(delim_bytes)?;
                            writer.write_all(lbl.as_bytes())?;
                            writer.write_all(b"\n")?;
                        } else {
                            writer.write_all(&column_names_bytes)?;
                            writer.write_all(b"\n")?;
                        }
                        if config.line_buffered {
                            writer.flush()?;
                        }
                        header_written = true;
                    }

                    let tests = build_tests(
                        Some(&column_names_bytes),
                        config.delimiter,
                        config.as_spec_config(),
                    )
                    .map_err(|e| anyhow::anyhow!(e))?;
                    tests_with_header = Some(tests);
                } else {
                    // HashLines mode: no column names line, but we still need to process data
                    // Use numeric field indices (no header for field name resolution)
                    let tests =
                        build_tests(None, config.delimiter, config.as_spec_config())
                            .map_err(|e| anyhow::anyhow!(e))?;
                    tests_with_header = Some(tests);

                    // Write the last hash line as header if present
                    if !header_written && !config.count_only {
                        if let Some(last_hash_line) = header_info.lines.last() {
                            if let Some(ref lbl) = config.label_header {
                                writer.write_all(last_hash_line)?;
                                writer.write_all(delim_bytes)?;
                                writer.write_all(lbl.as_bytes())?;
                                writer.write_all(b"\n")?;
                            } else {
                                writer.write_all(last_hash_line)?;
                                writer.write_all(b"\n")?;
                            }
                            if config.line_buffered {
                                writer.flush()?;
                            }
                            header_written = true;
                        }
                    }
                }
            } else {
                // No header found (e.g., HashLines mode with no hash lines)
                // Fall back to no-header mode for this file
                // Build tests without header for this file
                let tests = build_tests(None, config.delimiter, config.as_spec_config())
                    .map_err(|e| anyhow::anyhow!(e))?;
                tests_with_header = Some(tests);
            }
        }

        tsv_reader.for_each_row(delim_byte, |row: &TsvRow| {
            let tests: &[TestKind] = if has_header {
                match tests_with_header.as_ref() {
                    Some(v) => v.as_slice(),
                    None => return Ok(()),
                }
            } else {
                tests_without_header.as_ref().unwrap().as_slice()
            };

            let mut row_match = if tests.is_empty() {
                true
            } else if config.use_or {
                let mut any = false;
                for t in tests {
                    if t.eval_row(row) {
                        any = true;
                        break;
                    }
                }
                any
            } else {
                let mut all = true;
                for t in tests {
                    if !t.eval_row(row) {
                        all = false;
                        break;
                    }
                }
                all
            };

            if config.invert {
                row_match = !row_match;
            }

            if config.label_header.is_some() {
                let val = if row_match {
                    &config.label_pass_val
                } else {
                    &config.label_fail_val
                };
                writer.write_all(row.line)?;
                writer.write_all(delim_bytes)?;
                writer.write_all(val.as_bytes())?;
                writer.write_all(b"\n")?;
                if config.line_buffered {
                    writer.flush()?;
                }
            } else if row_match {
                if config.count_only {
                    total_matched += 1;
                } else {
                    writer.write_all(row.line)?;
                    writer.write_all(b"\n")?;
                    if config.line_buffered {
                        writer.flush()?;
                    }
                }
            }

            Ok(())
        })?;
    }

    if config.count_only {
        writeln!(writer, "{}", total_matched)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::filter::config::{NumericOp, PendingNumeric};
    use crate::libs::tsv::header::HeaderConfig;
    use tempfile::NamedTempFile;

    // Helper to create a temp file
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_basic_filter_numeric_gt() {
        let file = create_temp_file("1\t10\n2\t20\n3\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        // Spec: 2:20 (col 2 > 20)
        config.numeric_specs.push(PendingNumeric {
            spec: "2:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "3\t30\n");
    }

    #[test]
    fn test_filter_with_header_by_name() {
        let file = create_temp_file("ID\tValue\n1\t10\n2\t20\n3\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.header_config = HeaderConfig::new().enabled();
        // Spec: Value:20 (Value > 20)
        config.numeric_specs.push(PendingNumeric {
            spec: "Value:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        // Should include header
        assert_eq!(out_str, "ID\tValue\n3\t30\n");
    }

    #[test]
    fn test_filter_invert() {
        let file = create_temp_file("1\t10\n2\t20\n3\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.invert = true;
        // Spec: 2:20 (col 2 > 20)
        // Invert -> col 2 <= 20
        config.numeric_specs.push(PendingNumeric {
            spec: "2:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "1\t10\n2\t20\n");
    }

    #[test]
    fn test_filter_or_logic() {
        let file = create_temp_file("1\t10\n2\t20\n3\t30\n4\t40\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.use_or = true;
        // Col 1 == 1 OR Col 2 > 30
        config.numeric_specs.push(PendingNumeric {
            spec: "1:1".to_string(),
            op: NumericOp::Eq,
        });
        config.numeric_specs.push(PendingNumeric {
            spec: "2:30".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        // 1 matches first condition
        // 4 matches second condition (40 > 30)
        // 2 matches neither
        // 3 matches neither (30 is not > 30)
        assert_eq!(out_str, "1\t10\n4\t40\n");
    }

    #[test]
    fn test_filter_count_only() {
        let file = create_temp_file("1\t10\n2\t20\n3\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.count_only = true;
        // Spec: 2:10 (col 2 > 10) -> matches 20, 30
        config.numeric_specs.push(PendingNumeric {
            spec: "2:10".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "2\n");
    }

    #[test]
    fn test_filter_label_header() {
        let file = create_temp_file("ID\tValue\n1\t10\n2\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.header_config = HeaderConfig::new().enabled();
        config.label_header = Some("Label".to_string());
        config.label_pass_val = "PASS".to_string();
        config.label_fail_val = "FAIL".to_string();

        // Spec: Value > 20
        config.numeric_specs.push(PendingNumeric {
            spec: "Value:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        // Header should have Label
        // Row 1 (10) fails -> FAIL
        // Row 2 (30) passes -> PASS
        let expected = "ID\tValue\tLabel\n1\t10\tFAIL\n2\t30\tPASS\n";
        assert_eq!(out_str, expected);
    }

    #[test]
    fn test_filter_empty_specs_match_all() {
        let file = create_temp_file("1\n2\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        // No specs

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "1\n2\n");
    }

    #[test]
    fn test_filter_line_buffered() {
        let file = create_temp_file("1\n2\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.line_buffered = true;
        // No specs - match all

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "1\n2\n");
    }

    #[test]
    fn test_filter_header_without_label() {
        let file = create_temp_file("ID\tValue\n1\t10\n2\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.header_config = HeaderConfig::new().enabled();
        // label_header is None - should write header without label column

        // Spec: Value > 20
        config.numeric_specs.push(PendingNumeric {
            spec: "Value:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        // Header should be written without label
        assert_eq!(out_str, "ID\tValue\n2\t30\n");
    }

    #[test]
    fn test_filter_header_only_file() {
        // File with only header, no data rows
        let file = create_temp_file("ID\tValue\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.header_config = HeaderConfig::new().enabled();
        config.numeric_specs.push(PendingNumeric {
            spec: "Value:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        // Only header should be output
        assert_eq!(out_str, "ID\tValue\n");
    }

    #[test]
    fn test_filter_count_only_with_match() {
        let file = create_temp_file("1\t10\n2\t20\n3\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.count_only = true;
        // Spec: 2:10 (col 2 > 10) -> matches 20, 30
        config.numeric_specs.push(PendingNumeric {
            spec: "2:10".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "2\n");
    }

    #[test]
    fn test_filter_count_only_no_match() {
        let file = create_temp_file("1\t10\n2\t20\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.count_only = true;
        // Spec: 2:100 (col 2 > 100) -> no matches
        config.numeric_specs.push(PendingNumeric {
            spec: "2:100".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert_eq!(out_str, "0\n");
    }

    #[test]
    fn test_filter_label_header_line_buffered() {
        let file = create_temp_file("ID\tValue\n1\t10\n2\t30\n");
        let path = file.path().to_str().unwrap().to_string();

        let mut config = FilterConfig::default();
        config.delimiter = '\t';
        config.header_config = HeaderConfig::new().enabled();
        config.label_header = Some("Label".to_string());
        config.label_pass_val = "PASS".to_string();
        config.label_fail_val = "FAIL".to_string();
        config.line_buffered = true;

        // Spec: Value > 20
        config.numeric_specs.push(PendingNumeric {
            spec: "Value:20".to_string(),
            op: NumericOp::Gt,
        });

        let mut output = Vec::new();
        run_filter(&[path], &mut output, config).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let expected = "ID\tValue\tLabel\n1\t10\tFAIL\n2\t30\tPASS\n";
        assert_eq!(out_str, expected);
    }
}
