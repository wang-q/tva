use crate::libs::io::map_io_err;
use crate::libs::stats::Aggregator;
use crate::libs::tsv::fields;
use crate::libs::tsv::fields::Header;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::Row;
use clap::*;
use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("wider")
        .about("Reshapes a long table into a wide format (pivot table)")
        .after_help(include_str!("../../docs/help/wider.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("names-from")
                .long("names-from")
                .required(true)
                .help("Column(s) containing the new column headers"),
        )
        .arg(
            Arg::new("values-from")
                .long("values-from")
                .help("Column(s) containing the data values"),
        )
        .arg(
            Arg::new("id-cols")
                .long("id-cols")
                .help("Columns that identify each row (default: all others)"),
        )
        .arg(
            Arg::new("values-fill")
                .long("values-fill")
                .default_value("")
                .help("Value to use for missing cells"),
        )
        .arg(
            Arg::new("names-sort")
                .long("names-sort")
                .action(ArgAction::SetTrue)
                .help("Sort the resulting column headers"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("op")
                .long("op")
                .default_value("last")
                .value_parser([
                    "count", "sum", "mean", "min", "max", "first", "last", "median",
                    "q1", "q3", "iqr", "geomean", "harmmean", "cv", "range", "mode",
                    "stdev", "variance",
                ])
                .help("Aggregation operation to perform on value column"),
        )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Count,
    Sum,
    Mean,
    Min,
    Max,
    First,
    Last,
    Median,
    Q1,
    Q3,
    Iqr,
    GeoMean,
    HarmMean,
    CV,
    Range,
    Mode,
    Stdev,
    Variance,
}

struct Cell {
    count: usize,
    sum: f64,
    sum_sq: f64,
    sum_log: f64,
    sum_inv: f64,
    min: f64,
    max: f64,
    first: Option<Vec<u8>>,
    last: Option<Vec<u8>>,
    values: Vec<f64>,
    value_counts: HashMap<Vec<u8>, usize>, // For Mode
}

impl Cell {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            sum_log: 0.0,
            sum_inv: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            first: None,
            last: None,
            values: Vec::new(),
            value_counts: HashMap::new(),
        }
    }

    fn update(&mut self, val_bytes: &[u8], op: Op) {
        self.count += 1;

        if self.first.is_none() {
            self.first = Some(val_bytes.to_vec());
        }
        self.last = Some(val_bytes.to_vec());

        // Mode needs raw bytes
        if op == Op::Mode {
            *self.value_counts.entry(val_bytes.to_vec()).or_insert(0) += 1;
        }

        // Parse float if needed
        let val_opt = if matches!(op, Op::Count | Op::First | Op::Last | Op::Mode) {
             None
        } else {
             // Only parse if we need numerical value
             // Try to parse from bytes
             // We can use simd-json or fast-float if available, but std is fine for now
             if let Ok(s) = std::str::from_utf8(val_bytes) {
                 s.trim().parse::<f64>().ok()
             } else {
                 None
             }
        };

        if let Some(val) = val_opt {
            match op {
                Op::Sum | Op::Mean | Op::CV | Op::Stdev | Op::Variance => {
                    self.sum += val;
                    if matches!(op, Op::CV | Op::Stdev | Op::Variance) {
                        self.sum_sq += val * val;
                    }
                }
                Op::Min | Op::Range => {
                    if val < self.min {
                        self.min = val;
                    }
                    // Range needs max too
                    if op == Op::Range && val > self.max {
                        self.max = val;
                    }
                }
                Op::Max => {
                    if val > self.max {
                        self.max = val;
                    }
                }
                Op::GeoMean => {
                    if val > 0.0 {
                        self.sum_log += val.ln();
                    }
                }
                Op::HarmMean => {
                    if val != 0.0 {
                        self.sum_inv += 1.0 / val;
                    }
                }
                Op::Median | Op::Q1 | Op::Q3 | Op::Iqr => {
                    self.values.push(val);
                }
                _ => {}
            }

            if matches!(op, Op::Min | Op::Range) && val < self.min {
                self.min = val;
            }
            if matches!(op, Op::Max | Op::Range) && val > self.max {
                self.max = val;
            }
        }
    }

    fn result(&self, op: Op) -> String {
        match op {
            Op::Count => self.count.to_string(),
            Op::Sum => self.sum.to_string(),
            Op::Mean => {
                if self.count > 0 {
                    (self.sum / self.count as f64).to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::Min => {
                if self.min == f64::INFINITY {
                    "nan".to_string()
                } else {
                    self.min.to_string()
                }
            }
            Op::Max => {
                if self.max == f64::NEG_INFINITY {
                    "nan".to_string()
                } else {
                    self.max.to_string()
                }
            }
            Op::First => self.first.as_ref().map(|v| String::from_utf8_lossy(v).to_string()).unwrap_or_default(),
            Op::Last => self.last.as_ref().map(|v| String::from_utf8_lossy(v).to_string()).unwrap_or_default(),
            Op::GeoMean => {
                if self.count > 0 {
                    (self.sum_log / self.count as f64).exp().to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::HarmMean => {
                if self.count > 0 && self.sum_inv != 0.0 {
                    (self.count as f64 / self.sum_inv).to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::Range => {
                if self.min != f64::INFINITY && self.max != f64::NEG_INFINITY {
                    (self.max - self.min).to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::CV => {
                if self.count > 1 {
                    let mean = self.sum / self.count as f64;
                    let variance = (self.sum_sq
                        - (self.sum * self.sum) / self.count as f64)
                        / (self.count as f64 - 1.0);
                    let stdev = variance.sqrt();
                    if mean != 0.0 {
                        (stdev / mean).to_string()
                    } else {
                        "nan".to_string()
                    }
                } else {
                    "nan".to_string()
                }
            }
            Op::Stdev => {
                if self.count > 1 {
                    let variance = (self.sum_sq
                        - (self.sum * self.sum) / self.count as f64)
                        / (self.count as f64 - 1.0);
                    variance.sqrt().to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::Variance => {
                if self.count > 1 {
                    let variance = (self.sum_sq
                        - (self.sum * self.sum) / self.count as f64)
                        / (self.count as f64 - 1.0);
                    variance.to_string()
                } else {
                    "nan".to_string()
                }
            }
            Op::Median | Op::Q1 | Op::Q3 | Op::Iqr => {
                let mut sorted_vals = self.values.clone();
                sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                match op {
                    Op::Median => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.5).to_string()
                    }
                    Op::Q1 => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.25).to_string()
                    }
                    Op::Q3 => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.75).to_string()
                    }
                    Op::Iqr => {
                        let q1 = Aggregator::calculate_quantile(&sorted_vals, 0.25);
                        let q3 = Aggregator::calculate_quantile(&sorted_vals, 0.75);
                        (q3 - q1).to_string()
                    }
                    _ => unreachable!(),
                }
            }
            Op::Mode => {
                if self.value_counts.is_empty() {
                    "".to_string()
                } else {
                    let mut count_vec: Vec<(&Vec<u8>, &usize)> =
                        self.value_counts.iter().collect();
                    count_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                    String::from_utf8_lossy(count_vec[0].0).to_string()
                }
            }
        }
    }
}

struct WiderConfig {
    names_from: String,
    values_from: Option<String>,
    id_cols: Option<String>,
    fill_value: String,
    sort_names: bool,
    op: Op,
}

struct ProcessState {
    // Key: ID columns values
    // Value: Map of Name -> Cell
    data: IndexMap<Vec<u8>, IndexMap<Vec<u8>, Cell>>,
    all_names: IndexSet<Vec<u8>>,

    // Indices (0-based)
    names_idx: usize,
    values_idx: Option<usize>,
    id_indices: Vec<usize>,

    // Header info
    header_processed: bool,
    output_header_prefix: Vec<String>,
    first_file_header_len: usize,
}

impl ProcessState {
    fn new() -> Self {
        Self {
            data: IndexMap::new(),
            all_names: IndexSet::new(),
            names_idx: 0,
            values_idx: None,
            id_indices: Vec::new(),
            header_processed: false,
            output_header_prefix: Vec::new(),
            first_file_header_len: 0,
        }
    }
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let op_str = args.get_one::<String>("op").unwrap();
    let op = match op_str.as_str() {
        "count" => Op::Count,
        "sum" => Op::Sum,
        "mean" => Op::Mean,
        "min" => Op::Min,
        "max" => Op::Max,
        "first" => Op::First,
        "last" => Op::Last,
        "median" => Op::Median,
        "q1" => Op::Q1,
        "q3" => Op::Q3,
        "iqr" => Op::Iqr,
        "geomean" => Op::GeoMean,
        "harmmean" => Op::HarmMean,
        "cv" => Op::CV,
        "range" => Op::Range,
        "mode" => Op::Mode,
        "stdev" => Op::Stdev,
        "variance" => Op::Variance,
        _ => unreachable!(),
    };

    let values_from = args.get_one::<String>("values-from").cloned();
    if values_from.is_none() && op != Op::Count {
        anyhow::bail!("--values-from is required for operations other than count");
    }

    let config = WiderConfig {
        names_from: args.get_one::<String>("names-from").unwrap().clone(),
        values_from,
        id_cols: args.get_one::<String>("id-cols").cloned(),
        fill_value: args.get_one::<String>("values-fill").unwrap().clone(),
        sort_names: args.get_flag("names-sort"),
        op,
    };

    let mut state = ProcessState::new();

    for input in crate::libs::io::raw_input_sources(&infiles) {
         process_file(input, &config, &mut state)?;
    }

    write_output(&mut writer, &state, &config)?;

    Ok(())
}

fn process_file(
    input: crate::libs::io::InputSourceRaw,
    config: &WiderConfig,
    state: &mut ProcessState,
) -> anyhow::Result<()> {
    let mut tsv_reader = TsvReader::with_capacity(input.reader, 512 * 1024);

    // Read header
    let header_bytes = if let Some(h) = tsv_reader.read_header().map_err(map_io_err)? {
        h
    } else {
        return Ok(());
    };

    let line_str = String::from_utf8_lossy(&header_bytes);
    let header = Header::from_line(&line_str, '\t');
    let header_fields = &header.fields;

    if !state.header_processed {
        // Determine indices
        let n_indices = fields::parse_field_list_with_header(
            &config.names_from,
            Some(&header),
            '\t',
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        if n_indices.len() != 1 {
            return Err(anyhow::anyhow!(
                "Currently only single column supported for --names-from"
            ));
        }
        state.names_idx = n_indices[0] - 1;

        if let Some(v_spec) = &config.values_from {
            let v_indices =
                fields::parse_field_list_with_header(v_spec, Some(&header), '\t')
                    .map_err(|e| anyhow::anyhow!(e))?;
            if v_indices.len() != 1 {
                return Err(anyhow::anyhow!(
                    "Currently only single column supported for --values-from"
                ));
            }
            state.values_idx = Some(v_indices[0] - 1);
        } else {
            state.values_idx = None;
        }

        if let Some(spec) = &config.id_cols {
            let i_indices =
                fields::parse_field_list_with_header(spec, Some(&header), '\t')
                    .map_err(|e| anyhow::anyhow!(e))?;
            state.id_indices = i_indices.iter().map(|&i| i - 1).collect();
        } else {
            // Default: all except names and values
            for (i, _) in header_fields.iter().enumerate() {
                let is_val_idx = if let Some(idx) = state.values_idx {
                    i == idx
                } else {
                    false
                };
                if i != state.names_idx && !is_val_idx {
                    state.id_indices.push(i);
                }
            }
        }

        // Store output header prefix (ID column names)
        for &i in &state.id_indices {
            state.output_header_prefix.push(header_fields[i].clone());
        }

        state.first_file_header_len = header_fields.len();
        state.header_processed = true;
    } else {
        // Validate subsequent file headers
        if header_fields.len() != state.first_file_header_len {
            return Err(anyhow::anyhow!(
                 "File '{}' has {} columns, but first file had {}. All files must have the same column structure.",
                 input.name, header_fields.len(), state.first_file_header_len
             ));
        }
    }

    // Process rows
    tsv_reader.for_each_row(|row| {
        // Extract ID key
        let mut key = Vec::new();
        for (k_i, &idx) in state.id_indices.iter().enumerate() {
            if k_i > 0 {
                key.push(b'\t');
            }
            if let Some(field) = row.get_bytes(idx + 1) {
                key.extend_from_slice(field);
            }
        }

        // Extract Name
        let name = row.get_bytes(state.names_idx + 1).unwrap_or(&[]).to_vec();

        // Extract Value
        let value = if let Some(idx) = state.values_idx {
            row.get_bytes(idx + 1).unwrap_or(&[])
        } else {
            &[]
        };

        state.all_names.insert(name.clone());

        state
            .data
            .entry(key)
            .or_default()
            .entry(name)
            .or_insert_with(Cell::new)
            .update(value, config.op);
            
        Ok(())
    }).map_err(map_io_err)?;

    Ok(())
}

fn write_output<W: Write>(
    writer: &mut W,
    state: &ProcessState,
    config: &WiderConfig,
) -> anyhow::Result<()> {
    // Sort names if requested
    let final_names: Vec<Vec<u8>> = if config.sort_names {
        let mut sorted: Vec<Vec<u8>> = state.all_names.iter().cloned().collect();
        sorted.sort();
        sorted
    } else {
        state.all_names.iter().cloned().collect()
    };

    // Write Header
    for (i, col) in state.output_header_prefix.iter().enumerate() {
        if i > 0 {
            writer.write_all(b"\t")?;
        }
        writer.write_all(col.as_bytes())?;
    }
    for name in &final_names {
        if !state.output_header_prefix.is_empty() {
            writer.write_all(b"\t")?;
        }
        writer.write_all(name)?;
    }
    writer.write_all(b"\n")?;

    // Write Rows
    for (key, row_map) in &state.data {
        // Write ID cols (key already contains tabs if multiple ids)
        writer.write_all(key)?;

        // Write Value cols
        for name in &final_names {
            if !key.is_empty() {
                writer.write_all(b"\t")?;
            }
            if let Some(cell) = row_map.get(name) {
                writer.write_all(cell.result(config.op).as_bytes())?;
            } else {
                writer.write_all(config.fill_value.as_bytes())?;
            }
        }
        writer.write_all(b"\n")?;
    }

    Ok(())
}
