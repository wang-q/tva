//! Helpers for formatting output.
//!
//! The main entry point is [`format_number`], which formats a `f64` using
//! thousands separators and a configurable number of decimal digits.
//!
//! Basic usage:
//!
//! ```
//! use tva::libs::fmt::format_number;
//!
//! assert_eq!(format_number(1234.5, 1), "1,234.5");
//! assert_eq!(format_number(-1000.0, 0), "-1,000");
//! ```

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
}
