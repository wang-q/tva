//! Data loading and parsing utilities for plot commands.
//!
//! Provides shared functionality for reading TSV data, parsing columns,
//! and extracting numeric values for plotting.

use anyhow::Result;
use indexmap::IndexMap;

use crate::libs::tsv::fields::{parse_field_list_with_header, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRecord};
use crate::libs::tsv::split::TsvSplitter;

/// Column specification for plot commands
pub struct ColumnSpec {
    pub indices: Vec<usize>,
    pub names: Vec<String>,
}

impl ColumnSpec {
    /// Get single column index (0-based), error if multiple or none
    pub fn single(&self) -> Result<usize> {
        if self.indices.is_empty() {
            return Err(anyhow::anyhow!("No column specified"));
        }
        if self.indices.len() > 1 {
            return Err(anyhow::anyhow!(
                "Expected single column, got {} columns",
                self.indices.len()
            ));
        }
        Ok(self.indices[0])
    }

    /// Get single column name, error if multiple or none
    pub fn single_name(&self) -> Result<&str> {
        if self.names.is_empty() {
            return Err(anyhow::anyhow!("No column specified"));
        }
        if self.names.len() > 1 {
            return Err(anyhow::anyhow!(
                "Expected single column, got {} columns",
                self.names.len()
            ));
        }
        Ok(&self.names[0])
    }
}

/// Parse column specification from string
pub fn parse_columns(
    spec: &str,
    header: Option<&Header>,
    headers: &[Vec<u8>],
) -> Result<ColumnSpec> {
    let indices = parse_field_list_with_header(spec, header, '\t')
        .map_err(|e| anyhow::anyhow!("Invalid column spec '{}': {}", spec, e))?;

    if indices.is_empty() {
        return Err(anyhow::anyhow!("No valid columns specified"));
    }

    // Convert to 0-based indices
    let indices: Vec<usize> = indices.iter().map(|&i| i - 1).collect();

    // Get column names
    let names: Vec<String> = indices
        .iter()
        .map(|&idx| {
            headers
                .get(idx)
                .map(|h| String::from_utf8_lossy(h).to_string())
                .unwrap_or_else(|| format!("col{}", idx + 1))
        })
        .collect();

    Ok(ColumnSpec { indices, names })
}

/// Parse single column specification
pub fn parse_single_column(
    spec: &str,
    header: Option<&Header>,
    headers: &[Vec<u8>],
) -> Result<(usize, String)> {
    let col = parse_columns(spec, header, headers)?;
    let idx = col.single()?;
    let name = col.single_name()?.to_string();
    Ok((idx, name))
}

/// Data point for scatter plots
pub type Point = (f64, f64);

/// Load scatter plot data from TSV
pub fn load_scatter_data<R: std::io::Read>(
    mut reader: TsvReader<R>,
    x_idx: usize,
    y_indices: &[usize],
    y_names: &[String],
    color_idx: Option<usize>,
    ignore_errors: bool,
) -> Result<IndexMap<String, Vec<Point>>> {
    let mut data: IndexMap<String, Vec<Point>> = IndexMap::new();
    let mut record = TsvRecord::new();

    reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

        // Parse X value
        let x_bytes = match record.get_bytes(x_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found", x_idx + 1),
                ));
            }
        };

        let x_val = match crate::libs::number::fast_parse_f64(x_bytes) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number",
                        String::from_utf8_lossy(x_bytes)
                    ),
                ));
            }
        };

        // Get color group if specified
        let color_group = if let Some(idx) = color_idx {
            match record.get_bytes(idx + 1) {
                Some(b) => Some(String::from_utf8_lossy(b).to_string()),
                None => {
                    if ignore_errors {
                        return Ok(());
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Color column {} not found", idx + 1),
                    ));
                }
            }
        } else {
            None
        };

        // Parse each Y column
        for (y_idx, y_name) in y_indices.iter().zip(y_names.iter()) {
            let y_bytes = match record.get_bytes(y_idx + 1) {
                Some(b) => b,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Column {} not found", y_name),
                    ));
                }
            };

            let y_val = match crate::libs::number::fast_parse_f64(y_bytes) {
                Some(v) => v,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Cannot parse '{}' as number in column {}",
                            String::from_utf8_lossy(y_bytes),
                            y_name
                        ),
                    ));
                }
            };

            // Build group key
            let group_key = match &color_group {
                Some(c) => {
                    if y_indices.len() > 1 {
                        format!("{}|{}", c, y_name)
                    } else {
                        c.clone()
                    }
                }
                None => y_name.clone(),
            };

            data.entry(group_key).or_default().push((x_val, y_val));
        }

        Ok(())
    })?;

    Ok(data)
}

/// Load single column numeric data from TSV
pub fn load_numeric_column<R: std::io::Read>(
    mut reader: TsvReader<R>,
    col_idx: usize,
    col_name: &str,
    ignore_errors: bool,
) -> Result<Vec<f64>> {
    let mut values: Vec<f64> = Vec::new();
    let mut record = TsvRecord::new();

    reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

        let bytes = match record.get_bytes(col_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found", col_name),
                ));
            }
        };

        match crate::libs::number::fast_parse_f64(bytes) {
            Some(v) => values.push(v),
            None => {
                if !ignore_errors {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Cannot parse '{}' as number in column {}",
                            String::from_utf8_lossy(bytes),
                            col_name
                        ),
                    ));
                }
            }
        }

        Ok(())
    })?;

    Ok(values)
}

/// Load box plot data from TSV
pub fn load_box_data<R: std::io::Read>(
    mut reader: TsvReader<R>,
    y_indices: &[usize],
    y_names: &[String],
    color_idx: Option<usize>,
    ignore_errors: bool,
) -> Result<IndexMap<String, Vec<f64>>> {
    let mut data: IndexMap<String, Vec<f64>> = IndexMap::new();
    let mut record = TsvRecord::new();

    reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

        // Get color group if specified
        let color_group = if let Some(idx) = color_idx {
            match record.get_bytes(idx + 1) {
                Some(b) => Some(String::from_utf8_lossy(b).to_string()),
                None => {
                    if ignore_errors {
                        return Ok(());
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Color column {} not found", idx + 1),
                    ));
                }
            }
        } else {
            None
        };

        // Parse each Y column
        for (y_idx, y_name) in y_indices.iter().zip(y_names.iter()) {
            let y_bytes = match record.get_bytes(y_idx + 1) {
                Some(b) => b,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Column {} not found", y_name),
                    ));
                }
            };

            let y_val = match crate::libs::number::fast_parse_f64(y_bytes) {
                Some(v) => v,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Cannot parse '{}' as number in column {}",
                            String::from_utf8_lossy(y_bytes),
                            y_name
                        ),
                    ));
                }
            };

            // Build group key
            let group_key = match &color_group {
                Some(c) => {
                    if y_indices.len() > 1 {
                        format!("{}|{}", c, y_name)
                    } else {
                        c.clone()
                    }
                }
                None => y_name.clone(),
            };

            data.entry(group_key).or_default().push(y_val);
        }

        Ok(())
    })?;

    Ok(data)
}

/// Load 2D binning data from TSV
pub fn load_bin2d_data<R: std::io::Read>(
    mut reader: TsvReader<R>,
    x_idx: usize,
    x_name: &str,
    y_idx: usize,
    y_name: &str,
    ignore_errors: bool,
) -> Result<(Vec<f64>, Vec<f64>)> {
    let mut x_values: Vec<f64> = Vec::new();
    let mut y_values: Vec<f64> = Vec::new();
    let mut record = TsvRecord::new();

    reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

        // Parse X value
        let x_bytes = match record.get_bytes(x_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found", x_name),
                ));
            }
        };

        let x_val = match crate::libs::number::fast_parse_f64(x_bytes) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(x_bytes),
                        x_name
                    ),
                ));
            }
        };

        // Parse Y value
        let y_bytes = match record.get_bytes(y_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found", y_name),
                ));
            }
        };

        let y_val = match crate::libs::number::fast_parse_f64(y_bytes) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(y_bytes),
                        y_name
                    ),
                ));
            }
        };

        x_values.push(x_val);
        y_values.push(y_val);

        Ok(())
    })?;

    Ok((x_values, y_values))
}

/// Build header for field parsing from raw headers
pub fn build_header(headers: &[Vec<u8>]) -> Option<Header> {
    if headers.is_empty() {
        None
    } else {
        let field_names: Vec<String> = headers
            .iter()
            .map(|h| String::from_utf8_lossy(h).to_string())
            .collect();
        Some(Header::from_fields(field_names))
    }
}

/// Read headers from TSV reader
pub fn read_headers<R: std::io::Read>(
    reader: &mut TsvReader<R>,
) -> Result<Vec<Vec<u8>>> {
    let header_line = reader.read_header()?;
    Ok(match header_line {
        Some(line) => TsvSplitter::new(&line, b'\t').map(|s| s.to_vec()).collect(),
        None => Vec::new(),
    })
}
