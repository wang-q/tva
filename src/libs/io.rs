use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::read::MultiGzDecoder;

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

