use super::builder::build_tests;
use super::config::FilterConfig;
use super::engine::TestKind;
use crate::libs::io::map_io_err;
use crate::libs::tsv::record::TsvRow;
use anyhow::Result;
use memchr::memchr_iter;
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

    let tests_without_header: Option<Vec<TestKind>> = if config.has_header {
        None
    } else {
        Some(
            build_tests(None, config.delimiter, config.as_spec_config())
                .map_err(|e| anyhow::anyhow!(e))?,
        )
    };

    let max_field_without_header: usize = tests_without_header
        .as_ref()
        .map(|tests| tests.iter().map(|t| t.max_field_index()).max().unwrap_or(0))
        .unwrap_or(0);

    for input in crate::libs::io::raw_input_sources(infiles) {
        let mut tsv_reader = crate::libs::tsv::reader::TsvReader::new(input.reader);
        let mut tests_with_header: Option<Vec<TestKind>> = None;
        let mut max_field_for_rows = max_field_without_header;
        let mut ends: Vec<usize> = Vec::new();

        if config.has_header {
            if let Some(header_bytes) = tsv_reader.read_header().map_err(map_io_err)? {
                let header_line =
                    std::str::from_utf8(&header_bytes).map_err(map_io_err)?;
                let header = crate::libs::tsv::fields::Header::from_line(
                    header_line,
                    config.delimiter,
                );

                if !header_written && !config.count_only {
                    if let Some(ref lbl) = config.label_header {
                        writer.write_all(&header_bytes)?;
                        writer.write_all(delim_bytes)?;
                        writer.write_all(lbl.as_bytes())?;
                        writer.write_all(b"\n")?;
                    } else {
                        writer.write_all(&header_bytes)?;
                        writer.write_all(b"\n")?;
                    }
                    if config.line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }

                let tests = build_tests(
                    Some(&header),
                    config.delimiter,
                    config.as_spec_config(),
                )
                .map_err(|e| anyhow::anyhow!(e))?;
                max_field_for_rows =
                    tests.iter().map(|t| t.max_field_index()).max().unwrap_or(0);
                tests_with_header = Some(tests);
            }
        }

        tsv_reader.for_each_record(|record| {
            let tests: &[TestKind] = if config.has_header {
                match tests_with_header.as_ref() {
                    Some(v) => v.as_slice(),
                    None => return Ok(()),
                }
            } else {
                tests_without_header.as_ref().unwrap().as_slice()
            };

            let mut row_match = if tests.is_empty() {
                true
            } else {
                ends.clear();
                if max_field_for_rows > 0 {
                    let mut count = 0usize;
                    for pos in memchr_iter(delim_byte, record) {
                        ends.push(pos);
                        count += 1;
                        if count >= max_field_for_rows {
                            break;
                        }
                    }
                }

                let row = TsvRow {
                    line: record,
                    ends: &ends,
                };

                if config.use_or {
                    let mut any = false;
                    for t in tests {
                        if t.eval_row(&row) {
                            any = true;
                            break;
                        }
                    }
                    any
                } else {
                    let mut all = true;
                    for t in tests {
                        if !t.eval_row(&row) {
                            all = false;
                            break;
                        }
                    }
                    all
                }
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
                writer.write_all(record)?;
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
                    writer.write_all(record)?;
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
