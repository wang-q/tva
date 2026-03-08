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
//! let lines = read_lines(path).unwrap();
//! assert!(!lines.is_empty());
//! ```
//!
//! Creating a writer to stdout:
//!
//! ```
//! use std::io::Write;
//! use tva::libs::io::writer;
//!
//! let mut w = writer("stdout").unwrap();
//! writeln!(w, "hello\tworld").unwrap();
//! ```

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

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
pub fn raw_reader(input: &str) -> io::Result<Box<dyn Read>> {
    if is_stdin_name(input) {
        Ok(Box::new(io::stdin()))
    } else {
        let path = Path::new(input);
        let file = File::open(path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("tva: could not open {}: {}", path.display(), err),
            )
        })?;

        if path.extension() == Some(OsStr::new("gz")) {
            Ok(Box::new(MultiGzDecoder::new(file)))
        } else {
            Ok(Box::new(file))
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
/// let mut r = reader(path).unwrap();
/// let mut s = String::new();
/// r.read_to_string(&mut s).unwrap();
/// assert!(s.contains("hello"));
/// ```
pub fn reader(input: &str) -> io::Result<Box<dyn BufRead>> {
    let reader: Box<dyn BufRead> = if is_stdin_name(input) {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let path = Path::new(input);
        let file = File::open(path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("tva: could not open {}: {}", path.display(), err),
            )
        })?;

        if path.extension() == Some(OsStr::new("gz")) {
            Box::new(BufReader::new(MultiGzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        }
    };

    Ok(reader)
}

/// Creates a list of input sources from filenames.
///
/// Each `InputSource` contains the filename, a flag indicating if it is stdin,
/// and a `BufRead` reader for the content.
pub fn input_sources(infiles: &[String]) -> io::Result<Vec<InputSource>> {
    infiles
        .iter()
        .map(|name| {
            let is_stdin = is_stdin_name(name);
            let reader = reader(name)?;
            Ok(InputSource {
                name: name.clone(),
                is_stdin,
                reader,
            })
        })
        .collect()
}

/// Creates a list of raw input sources from filenames.
pub fn raw_input_sources(infiles: &[String]) -> io::Result<Vec<InputSourceRaw>> {
    infiles
        .iter()
        .map(|name| {
            let is_stdin = is_stdin_name(name);
            let reader = raw_reader(name)?;
            Ok(InputSourceRaw {
                name: name.clone(),
                is_stdin,
                reader,
            })
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
    let mut reader = reader(input)?;
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
/// let lines = read_lines(path).unwrap();
/// assert_eq!(lines.len(), 2);
/// assert_eq!(lines[0], "line1");
/// ```
pub fn read_lines(input: &str) -> io::Result<Vec<String>> {
    let mut reader = reader(input)?;
    let mut s = String::new();
    reader.read_to_string(&mut s).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("tva: read error from {}: {}", input, err),
        )
    })?;
    Ok(s.lines().map(|s| s.to_string()).collect::<Vec<String>>())
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
/// let replaces = read_replaces(path).unwrap();
/// assert!(replaces.contains_key("key1"));
/// assert_eq!(replaces["key1"], vec!["val1", "val2"]);
/// ```
pub fn read_replaces(input: &str) -> io::Result<BTreeMap<String, Vec<String>>> {
    let mut replaces: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for line in read_lines(input)? {
        let mut fields: Vec<&str> = line.split('\t').collect();

        let left = fields.split_off(1);

        replaces.insert(
            fields[0].to_string(),
            left.iter().map(|s| (*s).to_string()).collect(),
        );
    }

    Ok(replaces)
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
///     let mut w = writer(path).unwrap();
///     writeln!(w, "result").unwrap();
/// }
/// // Verify content
/// let content = std::fs::read_to_string(path).unwrap();
/// assert!(content.contains("result"));
/// ```
pub fn writer(output: &str) -> io::Result<Box<dyn Write>> {
    let writer: Box<dyn Write> = if output == "stdout" {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        let file = File::create(output).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("tva: could not create {}: {}", output, err),
            )
        })?;
        Box::new(BufWriter::new(file))
    };

    Ok(writer)
}

/// A manager for multiple output file writers with LRU eviction.
///
/// This is useful when writing to many output files simultaneously,
/// but you want to limit the number of open file handles.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use tva::libs::io::FileWriterManager;
/// use tempfile::TempDir;
///
/// let dir = TempDir::new().unwrap();
/// let mut manager = FileWriterManager::new(dir.path(), 2);
///
/// // Write to file 0
/// let w = manager.get_writer(0, "file_", ".txt").unwrap();
/// writeln!(w, "hello").unwrap();
///
/// // Write to file 1
/// let w = manager.get_writer(1, "file_", ".txt").unwrap();
/// writeln!(w, "world").unwrap();
/// ```
pub struct FileWriterManager {
    dir: PathBuf,
    writers: indexmap::IndexMap<usize, BufWriter<File>>,
    initialized: std::collections::HashSet<usize>,
    max_open: usize,
}

impl FileWriterManager {
    /// Creates a new FileWriterManager.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory where output files will be created
    /// * `max_open` - Maximum number of files to keep open (0 = unlimited)
    pub fn new(dir: &Path, max_open: usize) -> Self {
        Self {
            dir: dir.to_path_buf(),
            writers: indexmap::IndexMap::new(),
            initialized: std::collections::HashSet::new(),
            max_open,
        }
    }

    /// Gets a writer for the specified file index.
    ///
    /// If the writer is already open, returns the existing writer.
    /// If not, opens the file (creating if necessary, appending if previously opened).
    ///
    /// # Arguments
    ///
    /// * `idx` - The file index (used to construct filename)
    /// * `prefix` - Filename prefix
    /// * `suffix` - Filename suffix
    pub fn get_writer(
        &mut self,
        idx: usize,
        prefix: &str,
        suffix: &str,
    ) -> io::Result<&mut BufWriter<File>> {
        // Check if already open
        if self.writers.contains_key(&idx) {
            // Move to end (MRU) if using LRU logic
            if self.max_open > 0 {
                if let Some((_, v)) = self.writers.shift_remove_entry(&idx) {
                    self.writers.insert(idx, v);
                }
            }
            return Ok(self.writers.get_mut(&idx).unwrap());
        }

        // Evict LRU if at capacity
        if self.max_open > 0 && self.writers.len() >= self.max_open {
            self.writers.shift_remove_index(0);
        }

        // Open the file
        let filename = format!("{}{}{}", prefix, idx, suffix);
        let path = self.dir.join(&filename);
        let is_append = self.initialized.contains(&idx);

        let file = if is_append {
            OpenOptions::new().create(true).append(true).open(&path)?
        } else {
            File::create(&path)?
        };

        self.initialized.insert(idx);
        self.writers.insert(idx, BufWriter::new(file));

        Ok(self.writers.get_mut(&idx).unwrap())
    }

    /// Flushes all open writers.
    pub fn flush_all(&mut self) -> io::Result<()> {
        for (_, writer) in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

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

        let mut r = reader(path).unwrap();
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

        let mut r = reader(&path).unwrap();
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

        let lines = read_lines(path).unwrap();
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

        let replaces = read_replaces(path).unwrap();
        assert_eq!(replaces.len(), 2);
        assert_eq!(replaces["k1"], vec!["v1", "v2"]);
        assert_eq!(replaces["k2"], vec!["v3"]);
    }

    #[test]
    fn test_writer_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        {
            let mut w = writer(path).unwrap();
            writeln!(w, "test output").unwrap();
        }

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("test output"));
    }

    #[test]
    fn test_input_sources() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "file1 content").unwrap();
        let path1 = file1.path().to_str().unwrap().to_string();

        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "file2 content").unwrap();
        let path2 = file2.path().to_str().unwrap().to_string();

        let inputs = vec![path1.clone(), "stdin".to_string(), path2.clone()];
        let sources = input_sources(&inputs).unwrap();

        assert_eq!(sources.len(), 3);
        assert!(!sources[0].is_stdin);
        assert!(sources[1].is_stdin);
        assert!(!sources[2].is_stdin);

        assert_eq!(sources[0].name, path1);
        assert_eq!(sources[1].name, "stdin");
        assert_eq!(sources[2].name, path2);
    }

    #[test]
    fn test_raw_input_sources() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "raw content").unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let inputs = vec![path.clone()];
        let sources = raw_input_sources(&inputs).unwrap();

        assert_eq!(sources.len(), 1);
        assert!(!sources[0].is_stdin);
        assert_eq!(sources[0].name, path);
    }

    #[test]
    fn test_raw_reader_gzip() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string() + ".gz";

        {
            let f = File::create(&path).unwrap();
            let mut e = GzEncoder::new(f, Compression::default());
            e.write_all(b"raw gzip content").unwrap();
        }

        let mut r = raw_reader(&path).unwrap();
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "raw gzip content");

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_read_lines_error() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string() + ".gz";

        // Write invalid gzip data
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"not a gzip file").unwrap();
        }

        let result = read_lines(&path);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("tva: read error from"));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_writer_error() {
        // Try to create a file in a non-existent directory
        let path = "non_existent_dir_12345/file.txt";
        let result = writer(path);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("tva: could not create"));
    }

    #[test]
    fn test_file_writer_manager_basic() {
        let dir = TempDir::new().unwrap();
        let mut manager = FileWriterManager::new(dir.path(), 0); // unlimited

        // Write to file 0
        let w = manager.get_writer(0, "file_", ".txt").unwrap();
        writeln!(w, "hello").unwrap();

        // Write to file 1
        let w = manager.get_writer(1, "file_", ".txt").unwrap();
        writeln!(w, "world").unwrap();

        // Flush all
        manager.flush_all().unwrap();

        // Verify content
        let content0 = std::fs::read_to_string(dir.path().join("file_0.txt")).unwrap();
        assert_eq!(content0, "hello\n");

        let content1 = std::fs::read_to_string(dir.path().join("file_1.txt")).unwrap();
        assert_eq!(content1, "world\n");
    }

    #[test]
    fn test_file_writer_manager_lru() {
        let dir = TempDir::new().unwrap();
        let mut manager = FileWriterManager::new(dir.path(), 2); // max 2 open files

        // Write to files 0, 1, 2 (0 should be evicted when 2 is opened)
        let w = manager.get_writer(0, "f", ".txt").unwrap();
        writeln!(w, "file0").unwrap();

        let w = manager.get_writer(1, "f", ".txt").unwrap();
        writeln!(w, "file1").unwrap();

        let w = manager.get_writer(2, "f", ".txt").unwrap();
        writeln!(w, "file2").unwrap();

        // Access file 0 again (should reopen)
        let w = manager.get_writer(0, "f", ".txt").unwrap();
        writeln!(w, "more").unwrap();

        manager.flush_all().unwrap();

        // Verify all content
        let content0 = std::fs::read_to_string(dir.path().join("f0.txt")).unwrap();
        assert_eq!(content0, "file0\nmore\n");

        let content1 = std::fs::read_to_string(dir.path().join("f1.txt")).unwrap();
        assert_eq!(content1, "file1\n");

        let content2 = std::fs::read_to_string(dir.path().join("f2.txt")).unwrap();
        assert_eq!(content2, "file2\n");
    }

    #[test]
    fn test_file_writer_manager_reopen() {
        let dir = TempDir::new().unwrap();
        let mut manager = FileWriterManager::new(dir.path(), 0);

        // Write to file 0
        let w = manager.get_writer(0, "out_", ".tsv").unwrap();
        writeln!(w, "line1").unwrap();

        // Close and reopen (should append)
        manager.writers.clear(); // Force close

        let w = manager.get_writer(0, "out_", ".tsv").unwrap();
        writeln!(w, "line2").unwrap();

        manager.flush_all().unwrap();

        let content = std::fs::read_to_string(dir.path().join("out_0.tsv")).unwrap();
        assert_eq!(content, "line1\nline2\n");
    }
}
