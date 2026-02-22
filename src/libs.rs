use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::read::MultiGzDecoder;
use intspan::IntSpan;


/// ```
/// use std::io::BufRead;
/// let reader = tva::libs::reader("tests/genome/S288c.chr.sizes");
/// let mut lines = vec![];
/// for line in reader.lines() {
///     lines.push(line.unwrap());
/// }
/// assert_eq!(lines.len(), 16);
///
/// let reader = tva::libs::reader("tests/genome/S288c.chr.sizes");
/// assert_eq!(reader.lines().collect::<Vec<_>>().len(), 16);
/// ```
pub fn reader(input: &str) -> Box<dyn BufRead> {
    let reader: Box<dyn BufRead> = if input == "stdin" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let path = Path::new(input);
        let file = match File::open(path) {
            Err(why) => panic!("could not open {}: {}", path.display(), why),
            Ok(file) => file,
        };

        if path.extension() == Some(OsStr::new("gz")) {
            Box::new(BufReader::new(MultiGzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        }
    };

    reader
}

/// ```
/// let lines = tva::libs::read_lines("tests/genome/S288c.chr.sizes");
/// assert_eq!(lines.len(), 16);
/// ```
pub fn read_lines(input: &str) -> Vec<String> {
    let mut reader = reader(input);
    let mut s = String::new();
    reader.read_to_string(&mut s).expect("Read error");
    s.lines().map(|s| s.to_string()).collect::<Vec<String>>()
}


/// ```
/// let replaces = tva::libs::read_replaces("tests/genome/S288c.chr.sizes");
/// assert_eq!(replaces.len(), 16);
/// assert_eq!(*replaces.get("II").unwrap().get(0).unwrap(), "813184");
/// ```
pub fn read_replaces(input: &str) -> BTreeMap<String, Vec<String>> {
    let mut replaces: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for line in read_lines(input) {
        let mut fields: Vec<&str> = line.split('\t').collect();

        let left = fields.split_off(1);

        replaces.insert(
            fields[0].to_string(),
            left.iter().map(|s| (*s).to_string()).collect(),
        );
    }

    replaces
}


pub fn writer(output: &str) -> Box<dyn Write> {
    let writer: Box<dyn Write> = if output == "stdout" {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        Box::new(BufWriter::new(File::create(output).unwrap()))
    };

    writer
}

pub fn fields_to_ints(s: &str) -> IntSpan {
    let mut ints = IntSpan::new();
    let parts: Vec<&str> = s.split(',').collect();
    for p in parts {
        ints.add_runlist(p);
    }

    ints
}

pub fn parse_numeric_field_list(spec: &str) -> Result<Vec<usize>, String> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut ints: Vec<i32> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }
        let intspan = IntSpan::from(part);
        for e in intspan.elements() {
            if e <= 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            ints.push(e);
        }
    }

    ints.sort_unstable();
    ints.dedup();

    Ok(ints.iter().map(|e| *e as usize).collect())
}

pub fn fields_to_idx(spec: &str) -> Vec<usize> {
    parse_numeric_field_list(spec).unwrap()
}

// rewrite from https://metacpan.org/dist/Number-Format/source/Format.pm
pub fn format_number(number: f64, decimal_digits: usize) -> String {
    // Handle negative numbers
    let sign = if number < 0.0 { -1 } else { 1 };
    let mut number = number.abs();
    number = round(number, decimal_digits); // Round off number

    // Split integer and decimal parts of the number
    let integer_part = number.trunc() as i64;
    let decimal_part = number.fract();

    // Add the commas (fixed as `,`)
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
    // Implement rounding logic
    (number * 10f64.powi(precision as i32)).round() / 10f64.powi(precision as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        // Test positive numbers
        assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
        assert_eq!(format_number(1000.0, 0), "1,000");
        assert_eq!(format_number(0.12345, 3), "0.123");

        // Test negative numbers
        assert_eq!(format_number(-9876543.21, 3), "-9,876,543.210");
        assert_eq!(format_number(-1000.0, 0), "-1,000");
        assert_eq!(format_number(-0.98765, 4), "-0.9877");

        // Test zero
        assert_eq!(format_number(0.0, 2), "0.00");
        assert_eq!(format_number(-0.0, 2), "0.00");

        // Test large numbers
        assert_eq!(format_number(1e10, 2), "10,000,000,000.00");
        assert_eq!(format_number(-1e10, 2), "-10,000,000,000.00");

        // Test decimal places
        assert_eq!(format_number(1234.56789, 3), "1,234.568");
        assert_eq!(format_number(1234.0, 5), "1,234.00000");
    }

    #[test]
    fn test_parse_numeric_field_list_basic() {
        let v = parse_numeric_field_list("1,3-5").unwrap();
        assert_eq!(v, vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_whitespace_and_dup() {
        let v = parse_numeric_field_list(" 2 , 2 , 4-5 ").unwrap();
        assert_eq!(v, vec![2, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_empty() {
        let v = parse_numeric_field_list("   ").unwrap();
        assert!(v.is_empty());
    }
}
