use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::read::MultiGzDecoder;

fn is_stdin_name(name: &str) -> bool {
    name == "stdin" || name == "-"
}

pub struct InputSource {
    pub name: String,
    pub is_stdin: bool,
    pub reader: Box<dyn BufRead>,
}

pub fn reader(input: &str) -> Box<dyn BufRead> {
    let reader: Box<dyn BufRead> = if is_stdin_name(input) {
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
        if !trimmed.is_empty() {
            return Ok(true);
        }
    }
}

pub fn read_lines(input: &str) -> Vec<String> {
    let mut reader = reader(input);
    let mut s = String::new();
    reader.read_to_string(&mut s).expect("Read error");
    s.lines().map(|s| s.to_string()).collect::<Vec<String>>()
}

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
