//! Numeric utilities: parsing and formatting.
//!
//! This module combines high-performance parsing (from bytes) and
//! flexible formatting for floating point numbers.

/// Trims ASCII whitespace from the beginning and end of a byte slice.
#[inline]
fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &bytes[start..end]
}

/// Parses a byte slice into an f64 using lexical's high-performance algorithm.
/// Returns None if parsing fails.
///
/// This handles trimming of whitespace automatically.
/// This is significantly faster than `str::parse::<f64>` and operates directly on bytes,
/// avoiding UTF-8 validation overhead.
#[inline]
pub fn fast_parse_f64(bytes: &[u8]) -> Option<f64> {
    let trimmed = trim_bytes(bytes);
    if trimmed.is_empty() {
        return None;
    }
    lexical::parse(trimmed).ok()
}

/// Formats a number with thousands separators and fixed decimal precision.
pub fn format_number(number: f64, decimal_digits: usize) -> String {
    let sign = if number < 0.0 { -1 } else { 1 };
    let mut number = number.abs();
    number = round(number, decimal_digits);

    let integer_part = number.trunc() as i64;
    let decimal_part = number.fract();

    let integer_str = integer_part.to_string();
    let formatted_integer = integer_str
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect::<String>();

    let decimal_str = format!("{:.1$}", decimal_part, decimal_digits)
        .trim_start_matches('0')
        .to_string();

    let result = if !decimal_str.is_empty() {
        format!("{}{}", formatted_integer, decimal_str)
    } else {
        formatted_integer
    };

    if sign < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// Formats a float with fixed precision, handling NaN and Inf.
/// Trims trailing zeros for cleaner output.
/// If precision is None, uses default formatting (full precision).
/// This is used by `stats` command.
pub fn format_float(val: f64, precision: Option<usize>) -> String {
    if val.is_nan() {
        "nan".to_string()
    } else if val.is_infinite() {
        if val.is_sign_positive() {
            "inf".to_string()
        } else {
            "-inf".to_string()
        }
    } else {
        match precision {
            Some(p) => {
                let s = format!("{:.1$}", val, p);
                if s.contains('.') {
                    let s = s.trim_end_matches('0');
                    if s.ends_with('.') {
                        s[..s.len() - 1].to_string()
                    } else {
                        s.to_string()
                    }
                } else {
                    s
                }
            }
            None => val.to_string(),
        }
    }
}

fn round(number: f64, precision: usize) -> f64 {
    (number * 10f64.powi(precision as i32)).round() / 10f64.powi(precision as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
        assert_eq!(format_number(1000.0, 0), "1,000");
        assert_eq!(format_number(0.12345, 3), "0.123");

        assert_eq!(format_number(-9876543.21, 3), "-9,876,543.210");
        assert_eq!(format_number(-1000.0, 0), "-1,000");
        assert_eq!(format_number(-0.98765, 4), "-0.9877");

        assert_eq!(format_number(0.0, 2), "0.00");
        assert_eq!(format_number(-0.0, 2), "0.00");

        assert_eq!(format_number(1e10, 2), "10,000,000,000.00");
        assert_eq!(format_number(-1e10, 2), "-10,000,000,000.00");

        assert_eq!(format_number(1234.56789, 3), "1,234.568");
        assert_eq!(format_number(1234.0, 5), "1,234.00000");
    }

    #[test]
    fn test_format_float() {
        assert_eq!(format_float(1.23456, Some(2)), "1.23");
        assert_eq!(format_float(1.23456, Some(4)), "1.2346");
        assert_eq!(format_float(10.00, Some(4)), "10");
        assert_eq!(format_float(10.50, Some(4)), "10.5");
        assert_eq!(format_float(f64::NAN, Some(2)), "nan");
        assert_eq!(format_float(f64::INFINITY, Some(2)), "inf");
        assert_eq!(format_float(f64::NEG_INFINITY, Some(2)), "-inf");
        assert_eq!(format_float(1.23456, None), "1.23456");
    }

    #[test]
    fn test_trim_bytes() {
        // Tests L12-13 and L15-16: trim_bytes whitespace handling
        assert_eq!(trim_bytes(b"  123  "), b"123");
        assert_eq!(trim_bytes(b"\t123\n"), b"123");
        assert_eq!(trim_bytes(b"123"), b"123");
        assert_eq!(trim_bytes(b"   "), b""); // All whitespace
        assert_eq!(trim_bytes(b""), b""); // Empty
    }

    #[test]
    fn test_fast_parse_f64_empty() {
        // Tests L30-31: fast_parse_f64 empty input check
        assert_eq!(fast_parse_f64(b""), None);
        assert_eq!(fast_parse_f64(b"   "), None); // Trimmed becomes empty
    }

    #[test]
    fn test_format_float_trimming() {
        // Tests L99-100: format_float trailing zero trimming logic
        // Case 1: "10.50" -> "10.5" (L99 executed, L100 not)
        assert_eq!(format_float(10.5, Some(2)), "10.5");

        // Case 2: "10.00" -> "10." -> "10" (L99 executed, L100 executed)
        assert_eq!(format_float(10.0, Some(2)), "10");

        // Case 3: "10" (no dot) -> "10" (L99 not reached because contains('.') is false?)
        // Wait, format!("{:.1$}", 10.0, 0) gives "10".
        // If precision is 0, format! might output integer string without dot.
        assert_eq!(format_float(10.123, Some(0)), "10");
    }
}
