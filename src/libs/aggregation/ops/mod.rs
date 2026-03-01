pub mod basic;
pub mod mean;
pub mod variance;
pub mod quantile;
pub mod set;
pub mod text;

use crate::libs::tsv::record::Row;

/// Helper to parse a float from a row at a given index
#[inline]
pub(crate) fn parse_float(row: &dyn Row, idx: usize) -> Option<f64> {
    let s = row.get_str(idx + 1)?;
    if s.is_empty() {
        return None;
    }
    s.trim().parse::<f64>().ok()
}

/// Helper to get a string from a row at a given index
#[inline]
pub(crate) fn get_str(row: &dyn Row, idx: usize) -> String {
    row.get_str(idx + 1).unwrap_or("").to_string()
}
