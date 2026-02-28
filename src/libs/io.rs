//! I/O helpers used across tva commands.
//!
//! This module provides thin wrappers around standard I/O to simplify
//! working with stdin/stdout, regular files, and gzip-compressed inputs.
//!
//! Basic usage examples:
//!
//! Reading all non-empty lines from a file:
//!
//! ```
//! use std::io::Write;
//! use tva::libs::io::read_lines;
//! use tempfile::NamedTempFile;
//!
//! let mut file = NamedTempFile::new().unwrap();
//! writeln!(file, "line1").unwrap();
//! let path = file.path().to_str().unwrap();
//!
//! let lines = read_lines(path);
//! assert!(!lines.is_empty());
//! ```
//!
//! Creating a writer to stdout:
//!
//! ```
//! use std::io::Write;
//! use tva::libs::io::writer;
//!
//! let mut w = writer("stdout");
//! writeln!(w, "hello\tworld").unwrap();
//! ```

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::read::MultiGzDecoder;

/// Maps any error that implements `ToString` to `std::io::Error`.
///
/// This is useful for converting library-specific errors (like `anyhow::Error` or `csv::Error`)
/// into standard I/O errors, typically with `ErrorKind::InvalidData`.
pub fn map_io_err<E: ToString>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

fn is_stdin_name(name: &str) -> bool {
    name == "stdin" || name == "-"
}

pub struct InputSource {
    pub name: String,
    pub is_stdin: bool,
    pub reader: Box<dyn BufRead>,
}

pub struct InputSourceRaw {
    pub name: String,
    pub is_stdin: bool,
    pub reader: Box<dyn Read>,
}

/// Opens a file or stdin for reading.
///
/// If `input` is "stdin" or "-", it reads from standard input.
/// If the file extension is ".gz", it transparently decompresses the content.
pub fn raw_reader(input: &str) -> Box<dyn Read> {
    if is_stdin_name(input) {
        Box::new(io::stdin())
    } else {
        let path = Path::new(input);
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("tva: could not open {}: {}", path.display(), err);
                std::process::exit(1);
            }
        };

        if path.extension() == Some(OsStr::new("gz")) {
            Box::new(MultiGzDecoder::new(file))
        } else {
            Box::new(file)
        }
    }
}

/// Opens a file or stdin for reading (buffered).
///
/// If `input` is "stdin" or "-", it reads from standard input.
/// If the file extension is ".gz", it transparently decompresses the content.
///
/// # Examples
///
/// ```
/// use std::io::{Read, Write};
/// use tva::libs::io::reader;
/// use tempfile::NamedTempFile;
///
/// let mut file = NamedTempFile::new().unwrap();
/// writeln!(file, "hello").unwrap();
/// let path = file.path().to_str().unwrap();
///
/// let mut r = reader(path);
/// let mut s = String::new();
/// r.read_to_string(&mut s).unwrap();
/// assert!(s.contains("hello"));
/// ```
pub fn reader(input: &str) -> Box<dyn BufRead> {
    let reader: Box<dyn BufRead> = if is_stdin_name(input) {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let path = Path::new(input);
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("tva: could not open {}: {}", path.display(), err);
                std::process::exit(1);
            }
        };

        if path.extension() == Some(OsStr::new("gz")) {
            Box::new(BufReader::new(MultiGzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        }
    };

    reader
}

/// Creates a list of input sources from filenames.
///
/// Each `InputSource` contains the filename, a flag indicating if it is stdin,
/// and a `BufRead` reader for the content.
pub fn input_sources(infiles: &[String]) -> Vec<InputSource> {
    infiles
        .iter()
        .map(|name| {
            let is_stdin = is_stdin_name(name);
            let reader = reader(name);
            InputSource {
                name: name.clone(),
                is_stdin,
                reader,
            }
        })
        .collect()
}

/// Creates a list of raw input sources from filenames.
pub fn raw_input_sources(infiles: &[String]) -> Vec<InputSourceRaw> {
    infiles
        .iter()
        .map(|name| {
            let is_stdin = is_stdin_name(name);
            let reader = raw_reader(name);
            InputSourceRaw {
                name: name.clone(),
                is_stdin,
                reader,
            }
        })
        .collect()
}

/// Checks if the input contains any non-empty line.
///
/// Returns `Ok(true)` if at least one line with non-whitespace characters is found.
/// It reads lines from the beginning until a non-empty line is found or EOF is reached.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use tva::libs::io::has_nonempty_line;
/// use tempfile::NamedTempFile;
///
/// let mut file = NamedTempFile::new().unwrap();
/// writeln!(file, "content").unwrap();
/// let path = file.path().to_str().unwrap();
///
/// assert!(has_nonempty_line(path).unwrap());
/// ```
pub fn has_nonempty_line(input: &str) -> io::Result<bool> {
    let mut reader = reader(input);
    let mut buf = String::new();

    loop {
        buf.clear();
        let n = BufRead::read_line(&mut *reader, &mut buf)?;
        if n == 0 {
            return Ok(false);
        }
        let trimmed = buf.trim_end_matches(&['\n', '\r'][..]);
        if !trimmed.trim().is_empty() {
            return Ok(true);
        }
    }
}

/// Reads all lines from a file into a vector of strings.
///
/// It reads the entire file content into memory and splits it by lines.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use tva::libs::io::read_lines;
/// use tempfile::NamedTempFile;
///
/// let mut file = NamedTempFile::new().unwrap();
/// writeln!(file, "line1").unwrap();
/// writeln!(file, "line2").unwrap();
/// let path = file.path().to_str().unwrap();
///
/// let lines = read_lines(path);
/// assert_eq!(lines.len(), 2);
/// assert_eq!(lines[0], "line1");
/// ```
pub fn read_lines(input: &str) -> Vec<String> {
    let mut reader = reader(input);
    let mut s = String::new();
    if let Err(err) = reader.read_to_string(&mut s) {
        eprintln!("tva: read error from {}: {}", input, err);
        std::process::exit(1);
    }
    s.lines().map(|s| s.to_string()).collect::<Vec<String>>()
}

/// Reads tab-separated key-values replacement pairs from a file.
///
/// Each line is treated as a record. The first field is the key, and subsequent fields are values.
/// Returns a map where the key maps to a vector of values.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use tva::libs::io::read_replaces;
/// use tempfile::NamedTempFile;
///
/// let mut file = NamedTempFile::new().unwrap();
/// writeln!(file, "key1\tval1\tval2").unwrap();
/// let path = file.path().to_str().unwrap();
///
/// let replaces = read_replaces(path);
/// assert!(replaces.contains_key("key1"));
/// assert_eq!(replaces["key1"], vec!["val1", "val2"]);
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

/// Opens a file or stdout for writing.
///
/// If `output` is "stdout", it writes to standard output.
/// Returns a boxed writer that implements `Write`.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use tva::libs::io::writer;
/// use tempfile::NamedTempFile;
///
/// let file = NamedTempFile::new().unwrap();
/// let path = file.path().to_str().unwrap();
///
/// {
///     let mut w = writer(path);
///     writeln!(w, "result").unwrap();
/// }
/// // Verify content
/// let content = std::fs::read_to_string(path).unwrap();
/// assert!(content.contains("result"));
/// ```
pub fn writer(output: &str) -> Box<dyn Write> {
    let writer: Box<dyn Write> = if output == "stdout" {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        let file = match File::create(output) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("tva: could not create {}: {}", output, err);
                std::process::exit(1);
            }
        };
        Box::new(BufWriter::new(file))
    };

    writer
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    #[test]
    fn test_is_stdin_name() {
        assert!(is_stdin_name("stdin"));
        assert!(is_stdin_name("-"));
        assert!(!is_stdin_name("file.txt"));
    }

    #[test]
    fn test_map_io_err() {
        let err = map_io_err("custom error");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "custom error");
    }

    #[test]
    fn test_reader_regular_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "hello").unwrap();
        let path = file.path().to_str().unwrap();

        let mut r = reader(path);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert!(s.contains("hello"));
    }

    #[test]
    fn test_reader_gzip_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string() + ".gz";
        
        {
            let f = File::create(&path).unwrap();
            let mut e = GzEncoder::new(f, Compression::default());
            e.write_all(b"compressed hello").unwrap();
        }

        let mut r = reader(&path);
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "compressed hello");
        
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_has_nonempty_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "\n   \ncontent").unwrap();
        let path = file.path().to_str().unwrap();

        assert!(has_nonempty_line(path).unwrap());
        
        let mut empty_file = NamedTempFile::new().unwrap();
        writeln!(empty_file, "\n   \n").unwrap();
        let empty_path = empty_file.path().to_str().unwrap();
        
        assert!(!has_nonempty_line(empty_path).unwrap());
    }

    #[test]
    fn test_read_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line1").unwrap();
        writeln!(file, "line2").unwrap();
        let path = file.path().to_str().unwrap();

        let lines = read_lines(path);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_read_replaces() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "k1\tv1\tv2").unwrap();
        writeln!(file, "k2\tv3").unwrap();
        let path = file.path().to_str().unwrap();

        let replaces = read_replaces(path);
        assert_eq!(replaces.len(), 2);
        assert_eq!(replaces["k1"], vec!["v1", "v2"]);
        assert_eq!(replaces["k2"], vec!["v3"]);
    }

    #[test]
    fn test_writer_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        {
            let mut w = writer(path);
            writeln!(w, "test output").unwrap();
        }

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("test output"));
    }
}
