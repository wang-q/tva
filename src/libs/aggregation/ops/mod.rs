pub mod basic;
pub mod mean;
pub mod quantile;
pub mod set;
pub mod text;
pub mod variance;

use crate::libs::number::fast_parse_f64;
use crate::libs::tsv::record::Row;

/// Helper to parse a float from a row at a given index
#[inline]
pub(crate) fn parse_float(
    row: &dyn Row,
    idx: usize,
    default: Option<f64>,
    exclude_missing: bool,
) -> Option<f64> {
    match row.get_bytes(idx + 1) {
        None => {
            if exclude_missing {
                None
            } else {
                default
            }
        }
        Some(bytes) if bytes.is_empty() => {
            if exclude_missing {
                None
            } else {
                default
            }
        }
        Some(bytes) => fast_parse_f64(bytes),
    }
}

/// Helper to get a string from a row at a given index
#[inline]
pub(crate) fn get_str(row: &dyn Row, idx: usize) -> String {
    row.get_str(idx + 1).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::tsv::record::TsvRecord;

    #[test]
    fn test_parse_float() {
        let mut row = TsvRecord::new();
        // Setup row: "10.5", "", "abc", "20"
        row.parse_line(b"10.5\t\tabc\t20", b'\t');

        // Test normal float
        assert_eq!(parse_float(&row, 0, None, false), Some(10.5));

        // Test empty string with default
        assert_eq!(parse_float(&row, 1, Some(0.0), false), Some(0.0));

        // Test empty string with exclude_missing=true (should ignore default)
        assert_eq!(parse_float(&row, 1, Some(0.0), true), None);

        // Test invalid float "abc" -> fast_parse_f64 returns None
        assert_eq!(parse_float(&row, 2, None, false), None);

        // Test out of bounds
        assert_eq!(parse_float(&row, 99, Some(-1.0), false), Some(-1.0));
        assert_eq!(parse_float(&row, 99, Some(-1.0), true), None);
    }

    #[test]
    fn test_get_str() {
        let mut row = TsvRecord::new();
        row.parse_line(b"hello\tworld", b'\t');

        assert_eq!(get_str(&row, 0), "hello");
        assert_eq!(get_str(&row, 1), "world");
        assert_eq!(get_str(&row, 99), ""); // Out of bounds -> empty string
    }
}
