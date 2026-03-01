pub mod basic;
pub mod mean;
pub mod quantile;
pub mod set;
pub mod text;
pub mod variance;

use crate::libs::parse::fast_parse_f64;
use crate::libs::tsv::record::Row;

/// Helper to parse a float from a row at a given index
#[inline]
pub(crate) fn parse_float(row: &dyn Row, idx: usize, default: Option<f64>) -> Option<f64> {
    match row.get_bytes(idx + 1) {
        None => default,
        Some(bytes) if bytes.is_empty() => default,
        Some(bytes) => fast_parse_f64(bytes),
    }
}

/// Helper to get a string from a row at a given index
#[inline]
pub(crate) fn get_str(row: &dyn Row, idx: usize) -> String {
    row.get_str(idx + 1).unwrap_or("").to_string()
}
