use crate::libs::tsv::record::Row;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpKind {
    Count,
    Sum,
    Mean,
    Min,
    Max,
    Median,
    Q1,
    Q3,
    IQR,
    Stdev,
    CV,
    Variance,
    Mad,
    Range,
    First,
    Last,
    NUnique,
    Mode,
    GeoMean,
    HarmMean,
    Unique,
    Collapse,
    Rand,
}

#[derive(Clone)]
pub struct Operation {
    pub kind: OpKind,
    pub field_idx: Option<usize>, // None for count
}

/// A cell that holds aggregated data for a single value.
/// This is used by `wider` command where each cell in the pivot table
/// is an independent aggregation.
pub struct Cell {
    pub count: usize,
    pub sum: f64,
    pub sum_sq: f64,
    pub sum_log: f64,
    pub sum_inv: f64,
    pub min: f64,
    pub max: f64,
    pub first: Option<Vec<u8>>,
    pub last: Option<Vec<u8>>,
    pub values: Vec<f64>,
    pub value_counts: HashMap<Vec<u8>, usize>, // For Mode
}

impl Cell {
    pub fn new() -> Self {
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

    pub fn update(&mut self, val_bytes: &[u8], op: OpKind) {
        self.count += 1;

        if self.first.is_none() {
            self.first = Some(val_bytes.to_vec());
        }
        self.last = Some(val_bytes.to_vec());

        // Mode needs raw bytes
        if op == OpKind::Mode {
            *self.value_counts.entry(val_bytes.to_vec()).or_insert(0) += 1;
        }

        // Parse float if needed
        let val_opt = if matches!(
            op,
            OpKind::Count | OpKind::First | OpKind::Last | OpKind::Mode
        ) {
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
                OpKind::Sum
                | OpKind::Mean
                | OpKind::CV
                | OpKind::Stdev
                | OpKind::Variance => {
                    self.sum += val;
                    if matches!(op, OpKind::CV | OpKind::Stdev | OpKind::Variance) {
                        self.sum_sq += val * val;
                    }
                }
                OpKind::Min | OpKind::Range => {
                    if val < self.min {
                        self.min = val;
                    }
                    // Range needs max too
                    if op == OpKind::Range && val > self.max {
                        self.max = val;
                    }
                }
                OpKind::Max => {
                    if val > self.max {
                        self.max = val;
                    }
                }
                OpKind::GeoMean => {
                    if val > 0.0 {
                        self.sum_log += val.ln();
                    }
                }
                OpKind::HarmMean => {
                    if val != 0.0 {
                        self.sum_inv += 1.0 / val;
                    }
                }
                OpKind::Median | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
                    self.values.push(val);
                }
                _ => {}
            }

            if matches!(op, OpKind::Min | OpKind::Range) && val < self.min {
                self.min = val;
            }
            if matches!(op, OpKind::Max | OpKind::Range) && val > self.max {
                self.max = val;
            }
        }
    }

    pub fn result(&self, op: OpKind) -> String {
        match op {
            OpKind::Count => self.count.to_string(),
            OpKind::Sum => self.sum.to_string(),
            OpKind::Mean => {
                if self.count > 0 {
                    (self.sum / self.count as f64).to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::Min => {
                if self.min == f64::INFINITY {
                    "nan".to_string()
                } else {
                    self.min.to_string()
                }
            }
            OpKind::Max => {
                if self.max == f64::NEG_INFINITY {
                    "nan".to_string()
                } else {
                    self.max.to_string()
                }
            }
            OpKind::First => self
                .first
                .as_ref()
                .map(|v| String::from_utf8_lossy(v).to_string())
                .unwrap_or_default(),
            OpKind::Last => self
                .last
                .as_ref()
                .map(|v| String::from_utf8_lossy(v).to_string())
                .unwrap_or_default(),
            OpKind::GeoMean => {
                if self.count > 0 {
                    (self.sum_log / self.count as f64).exp().to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::HarmMean => {
                if self.count > 0 && self.sum_inv != 0.0 {
                    (self.count as f64 / self.sum_inv).to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::Range => {
                if self.min != f64::INFINITY && self.max != f64::NEG_INFINITY {
                    (self.max - self.min).to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::CV => {
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
            OpKind::Stdev => {
                if self.count > 1 {
                    let variance = (self.sum_sq
                        - (self.sum * self.sum) / self.count as f64)
                        / (self.count as f64 - 1.0);
                    variance.sqrt().to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::Variance => {
                if self.count > 1 {
                    let variance = (self.sum_sq
                        - (self.sum * self.sum) / self.count as f64)
                        / (self.count as f64 - 1.0);
                    variance.to_string()
                } else {
                    "nan".to_string()
                }
            }
            OpKind::Median | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
                let mut sorted_vals = self.values.clone();
                sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                match op {
                    OpKind::Median => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.5).to_string()
                    }
                    OpKind::Q1 => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.25).to_string()
                    }
                    OpKind::Q3 => {
                        Aggregator::calculate_quantile(&sorted_vals, 0.75).to_string()
                    }
                    OpKind::IQR => {
                        let q1 = Aggregator::calculate_quantile(&sorted_vals, 0.25);
                        let q3 = Aggregator::calculate_quantile(&sorted_vals, 0.75);
                        (q3 - q1).to_string()
                    }
                    _ => unreachable!(),
                }
            }
            OpKind::Mode => {
                if self.value_counts.is_empty() {
                    "".to_string()
                } else {
                    let mut count_vec: Vec<(&Vec<u8>, &usize)> =
                        self.value_counts.iter().collect();
                    count_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                    String::from_utf8_lossy(count_vec[0].0).to_string()
                }
            }
            _ => "nan".to_string(), // Fallback for unimplemented
        }
    }
}

/// Plan for executing statistics.
/// It maps input fields to internal storage slots.
pub struct StatsProcessor {
    pub ops: Vec<Operation>,

    // Mappings from field_idx -> storage_idx
    pub sum_map: HashMap<usize, usize>,
    pub sum_sq_map: HashMap<usize, usize>,
    pub sum_log_map: HashMap<usize, usize>,
    pub sum_inv_map: HashMap<usize, usize>,
    pub min_map: HashMap<usize, usize>,
    pub max_map: HashMap<usize, usize>,
    pub count_map: HashMap<usize, usize>, // field counts (numeric)
    pub values_map: HashMap<usize, usize>,
    pub first_map: HashMap<usize, usize>,
    pub last_map: HashMap<usize, usize>,
    pub value_counts_map: HashMap<usize, usize>,
    pub string_values_map: HashMap<usize, usize>,

    // Sizes for storage
    pub num_sums: usize,
    pub num_sum_sqs: usize,
    pub num_sum_logs: usize,
    pub num_sum_invs: usize,
    pub num_mins: usize,
    pub num_maxs: usize,
    pub num_field_counts: usize,
    pub num_values: usize,
    pub num_firsts: usize,
    pub num_lasts: usize,
    pub num_value_counts: usize,
    pub num_string_values: usize,
}

impl StatsProcessor {
    pub fn new(ops: Vec<Operation>) -> Self {
        let ops_clone = ops.clone();
        let mut processor = Self {
            ops,
            sum_map: HashMap::new(),
            sum_sq_map: HashMap::new(),
            sum_log_map: HashMap::new(),
            sum_inv_map: HashMap::new(),
            min_map: HashMap::new(),
            max_map: HashMap::new(),
            count_map: HashMap::new(),
            values_map: HashMap::new(),
            first_map: HashMap::new(),
            last_map: HashMap::new(),
            value_counts_map: HashMap::new(),
            string_values_map: HashMap::new(),

            num_sums: 0,
            num_sum_sqs: 0,
            num_sum_logs: 0,
            num_sum_invs: 0,
            num_mins: 0,
            num_maxs: 0,
            num_field_counts: 0,
            num_values: 0,
            num_firsts: 0,
            num_lasts: 0,
            num_value_counts: 0,
            num_string_values: 0,
        };

        for op in ops_clone {
            if let Some(idx) = op.field_idx {
                match op.kind {
                    OpKind::Sum | OpKind::Mean | OpKind::CV => {
                        processor.add_sum(idx);
                        processor.add_count(idx);
                        if op.kind == OpKind::CV {
                            processor.add_sum_sq(idx);
                        }
                    }
                    OpKind::Stdev | OpKind::Variance => {
                        processor.add_sum(idx);
                        processor.add_sum_sq(idx);
                        processor.add_count(idx);
                    }
                    OpKind::GeoMean => {
                        processor.add_sum_log(idx);
                        processor.add_count(idx);
                    }
                    OpKind::HarmMean => {
                        processor.add_sum_inv(idx);
                        processor.add_count(idx);
                    }
                    OpKind::Min | OpKind::Range => {
                        processor.add_min(idx);
                        if op.kind == OpKind::Range {
                            processor.add_max(idx);
                        }
                    }
                    OpKind::Max => {
                        processor.add_max(idx);
                    }
                    OpKind::Median
                    | OpKind::Mad
                    | OpKind::Q1
                    | OpKind::Q3
                    | OpKind::IQR => {
                        processor.add_values(idx);
                    }
                    OpKind::First => {
                        processor.add_first(idx);
                    }
                    OpKind::Last => {
                        processor.add_last(idx);
                    }
                    OpKind::NUnique | OpKind::Mode | OpKind::Unique => {
                        processor.add_value_counts(idx);
                    }
                    OpKind::Collapse | OpKind::Rand => {
                        processor.add_string_values(idx);
                    }
                    _ => {}
                }
            }
        }
        processor
    }

    fn add_sum(&mut self, idx: usize) {
        if !self.sum_map.contains_key(&idx) {
            self.sum_map.insert(idx, self.num_sums);
            self.num_sums += 1;
        }
    }
    fn add_sum_sq(&mut self, idx: usize) {
        if !self.sum_sq_map.contains_key(&idx) {
            self.sum_sq_map.insert(idx, self.num_sum_sqs);
            self.num_sum_sqs += 1;
        }
    }
    fn add_sum_log(&mut self, idx: usize) {
        if !self.sum_log_map.contains_key(&idx) {
            self.sum_log_map.insert(idx, self.num_sum_logs);
            self.num_sum_logs += 1;
        }
    }
    fn add_sum_inv(&mut self, idx: usize) {
        if !self.sum_inv_map.contains_key(&idx) {
            self.sum_inv_map.insert(idx, self.num_sum_invs);
            self.num_sum_invs += 1;
        }
    }
    fn add_min(&mut self, idx: usize) {
        if !self.min_map.contains_key(&idx) {
            self.min_map.insert(idx, self.num_mins);
            self.num_mins += 1;
        }
    }
    fn add_max(&mut self, idx: usize) {
        if !self.max_map.contains_key(&idx) {
            self.max_map.insert(idx, self.num_maxs);
            self.num_maxs += 1;
        }
    }
    fn add_count(&mut self, idx: usize) {
        if !self.count_map.contains_key(&idx) {
            self.count_map.insert(idx, self.num_field_counts);
            self.num_field_counts += 1;
        }
    }
    fn add_values(&mut self, idx: usize) {
        if !self.values_map.contains_key(&idx) {
            self.values_map.insert(idx, self.num_values);
            self.num_values += 1;
        }
    }
    fn add_first(&mut self, idx: usize) {
        if !self.first_map.contains_key(&idx) {
            self.first_map.insert(idx, self.num_firsts);
            self.num_firsts += 1;
        }
    }
    fn add_last(&mut self, idx: usize) {
        if !self.last_map.contains_key(&idx) {
            self.last_map.insert(idx, self.num_lasts);
            self.num_lasts += 1;
        }
    }
    fn add_value_counts(&mut self, idx: usize) {
        if !self.value_counts_map.contains_key(&idx) {
            self.value_counts_map.insert(idx, self.num_value_counts);
            self.num_value_counts += 1;
        }
    }
    fn add_string_values(&mut self, idx: usize) {
        if !self.string_values_map.contains_key(&idx) {
            self.string_values_map.insert(idx, self.num_string_values);
            self.num_string_values += 1;
        }
    }

    pub fn create_aggregator(&self) -> Aggregator {
        Aggregator {
            count: 0,
            sums: vec![0.0; self.num_sums],
            sum_sqs: vec![0.0; self.num_sum_sqs],
            sum_logs: vec![0.0; self.num_sum_logs],
            sum_invs: vec![0.0; self.num_sum_invs],
            mins: vec![f64::INFINITY; self.num_mins],
            maxs: vec![f64::NEG_INFINITY; self.num_maxs],
            field_counts: vec![0; self.num_field_counts],
            values: vec![Vec::new(); self.num_values],
            firsts: vec![String::new(); self.num_firsts],
            lasts: vec![String::new(); self.num_lasts],
            value_counts: vec![HashMap::new(); self.num_value_counts],
            string_values: vec![Vec::new(); self.num_string_values],
        }
    }

    pub fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        agg.count += 1;

        // Sums
        for (&idx, &slot) in &self.sum_map {
            if let Some(val) = Self::parse_float(row, idx) {
                agg.sums[slot] += val;
                if let Some(&count_slot) = self.count_map.get(&idx) {
                    agg.field_counts[count_slot] += 1;
                }
            }
        }

        // Sum Sqs
        for (&idx, &slot) in &self.sum_sq_map {
            if let Some(val) = Self::parse_float(row, idx) {
                agg.sum_sqs[slot] += val * val;
            }
        }

        // Sum Logs (GeoMean)
        for (&idx, &slot) in &self.sum_log_map {
            if let Some(val) = Self::parse_float(row, idx) {
                if val > 0.0 {
                    agg.sum_logs[slot] += val.ln();
                    // Update count if not already updated by sums
                    if !self.sum_map.contains_key(&idx) {
                        if let Some(&count_slot) = self.count_map.get(&idx) {
                            agg.field_counts[count_slot] += 1;
                        }
                    }
                }
            }
        }

        // Sum Invs (HarmMean)
        for (&idx, &slot) in &self.sum_inv_map {
            if let Some(val) = Self::parse_float(row, idx) {
                if val != 0.0 {
                    agg.sum_invs[slot] += 1.0 / val;
                    // Update count if not updated by sum or log
                    if !self.sum_map.contains_key(&idx)
                        && !self.sum_log_map.contains_key(&idx)
                    {
                        if let Some(&count_slot) = self.count_map.get(&idx) {
                            agg.field_counts[count_slot] += 1;
                        }
                    }
                }
            }
        }

        // Mins
        for (&idx, &slot) in &self.min_map {
            if let Some(val) = Self::parse_float(row, idx) {
                if val < agg.mins[slot] {
                    agg.mins[slot] = val;
                }
            }
        }

        // Maxs
        for (&idx, &slot) in &self.max_map {
            if let Some(val) = Self::parse_float(row, idx) {
                if val > agg.maxs[slot] {
                    agg.maxs[slot] = val;
                }
            }
        }

        // Values (Median/Mad/Quantiles)
        for (&idx, &slot) in &self.values_map {
            if let Some(val) = Self::parse_float(row, idx) {
                agg.values[slot].push(val);
            }
        }

        // Firsts
        for (&idx, &slot) in &self.first_map {
            if agg.firsts[slot].is_empty() {
                agg.firsts[slot] = row.get_str(idx + 1).unwrap_or("").to_string();
            }
        }

        // Lasts
        for (&idx, &slot) in &self.last_map {
            agg.lasts[slot] = row.get_str(idx + 1).unwrap_or("").to_string();
        }

        // Value Counts
        for (&idx, &slot) in &self.value_counts_map {
            let val = row.get_str(idx + 1).unwrap_or("").to_string();
            *agg.value_counts[slot].entry(val).or_insert(0) += 1;
        }

        // String Values
        for (&idx, &slot) in &self.string_values_map {
            let val = row.get_str(idx + 1).unwrap_or("").to_string();
            agg.string_values[slot].push(val);
        }
    }

    fn parse_float(row: &dyn Row, idx: usize) -> Option<f64> {
        let s = row.get_str(idx + 1)?;
        if s.is_empty() {
            return None;
        }
        s.trim().parse::<f64>().ok()
    }

    pub fn format_results(&self, agg: &Aggregator) -> Vec<String> {
        let mut values = Vec::new();
        for op in &self.ops {
            match op.kind {
                OpKind::Count => values.push(agg.count.to_string()),
                OpKind::Sum => {
                    if let Some(idx) = op.field_idx {
                        let slot = self.sum_map[&idx];
                        values.push(agg.sums[slot].to_string());
                    }
                }
                OpKind::Mean => {
                    if let Some(idx) = op.field_idx {
                        let sum = agg.sums[self.sum_map[&idx]];
                        let count = agg.field_counts[self.count_map[&idx]];
                        if count > 0 {
                            values.push((sum / count as f64).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Min => {
                    if let Some(idx) = op.field_idx {
                        let val = agg.mins[self.min_map[&idx]];
                        if val == f64::INFINITY {
                            values.push("nan".to_string());
                        } else {
                            values.push(val.to_string());
                        }
                    }
                }
                OpKind::Max => {
                    if let Some(idx) = op.field_idx {
                        let val = agg.maxs[self.max_map[&idx]];
                        if val == f64::NEG_INFINITY {
                            values.push("nan".to_string());
                        } else {
                            values.push(val.to_string());
                        }
                    }
                }
                OpKind::Range => {
                    if let Some(idx) = op.field_idx {
                        let min = agg.mins[self.min_map[&idx]];
                        let max = agg.maxs[self.max_map[&idx]];
                        if min != f64::INFINITY && max != f64::NEG_INFINITY {
                            values.push((max - min).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Stdev | OpKind::Variance | OpKind::CV => {
                    if let Some(idx) = op.field_idx {
                        let sum = agg.sums[self.sum_map[&idx]];
                        let sum_sq = agg.sum_sqs[self.sum_sq_map[&idx]];
                        let count = agg.field_counts[self.count_map[&idx]];

                        if count > 1 {
                            let variance = (sum_sq - (sum * sum) / count as f64)
                                / (count as f64 - 1.0);
                            match op.kind {
                                OpKind::Variance => values.push(variance.to_string()),
                                OpKind::Stdev => {
                                    values.push(variance.sqrt().to_string())
                                }
                                OpKind::CV => {
                                    let mean = sum / count as f64;
                                    if mean != 0.0 {
                                        values
                                            .push((variance.sqrt() / mean).to_string());
                                    } else {
                                        values.push("nan".to_string());
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::GeoMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_log = agg.sum_logs[self.sum_log_map[&idx]];
                        let count = agg.field_counts[self.count_map[&idx]];
                        if count > 0 {
                            values.push((sum_log / count as f64).exp().to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::HarmMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_inv = agg.sum_invs[self.sum_inv_map[&idx]];
                        let count = agg.field_counts[self.count_map[&idx]];
                        if count > 0 && sum_inv != 0.0 {
                            values.push((count as f64 / sum_inv).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Median | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
                    if let Some(idx) = op.field_idx {
                        let vals = &agg.values[self.values_map[&idx]];
                        if !vals.is_empty() {
                            let mut sorted_vals = vals.clone();
                            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

                            match op.kind {
                                OpKind::Median => values.push(
                                    Aggregator::calculate_quantile(&sorted_vals, 0.5)
                                        .to_string(),
                                ),
                                OpKind::Q1 => values.push(
                                    Aggregator::calculate_quantile(&sorted_vals, 0.25)
                                        .to_string(),
                                ),
                                OpKind::Q3 => values.push(
                                    Aggregator::calculate_quantile(&sorted_vals, 0.75)
                                        .to_string(),
                                ),
                                OpKind::IQR => {
                                    let q1 = Aggregator::calculate_quantile(
                                        &sorted_vals,
                                        0.25,
                                    );
                                    let q3 = Aggregator::calculate_quantile(
                                        &sorted_vals,
                                        0.75,
                                    );
                                    values.push((q3 - q1).to_string());
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Mad => {
                    if let Some(idx) = op.field_idx {
                        let vals = &agg.values[self.values_map[&idx]];
                        if !vals.is_empty() {
                            let mut sorted_vals = vals.clone();
                            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let len = sorted_vals.len();
                            let median = if len % 2 == 1 {
                                sorted_vals[len / 2]
                            } else {
                                let mid = len / 2;
                                (sorted_vals[mid - 1] + sorted_vals[mid]) / 2.0
                            };

                            let mut deviations: Vec<f64> =
                                vals.iter().map(|v| (v - median).abs()).collect();
                            deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

                            let mad = if len % 2 == 1 {
                                deviations[len / 2]
                            } else {
                                let mid = len / 2;
                                (deviations[mid - 1] + deviations[mid]) / 2.0
                            };
                            values.push(mad.to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::First => {
                    if let Some(idx) = op.field_idx {
                        values.push(agg.firsts[self.first_map[&idx]].clone());
                    }
                }
                OpKind::Last => {
                    if let Some(idx) = op.field_idx {
                        values.push(agg.lasts[self.last_map[&idx]].clone());
                    }
                }
                OpKind::NUnique => {
                    if let Some(idx) = op.field_idx {
                        values.push(
                            agg.value_counts[self.value_counts_map[&idx]]
                                .len()
                                .to_string(),
                        );
                    }
                }
                OpKind::Mode => {
                    if let Some(idx) = op.field_idx {
                        let counts = &agg.value_counts[self.value_counts_map[&idx]];
                        if counts.is_empty() {
                            values.push("".to_string());
                        } else {
                            let mut count_vec: Vec<(&String, &usize)> =
                                counts.iter().collect();
                            count_vec
                                .sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                            values.push(count_vec[0].0.clone());
                        }
                    }
                }
                OpKind::Unique => {
                    if let Some(idx) = op.field_idx {
                        let counts = &agg.value_counts[self.value_counts_map[&idx]];
                        if counts.is_empty() {
                            values.push("".to_string());
                        } else {
                            let mut keys: Vec<&String> = counts.keys().collect();
                            keys.sort();
                            values.push(
                                keys.into_iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<&str>>()
                                    .join(","),
                            );
                        }
                    }
                }
                OpKind::Collapse => {
                    if let Some(idx) = op.field_idx {
                        let vals = &agg.string_values[self.string_values_map[&idx]];
                        values.push(vals.join(","));
                    }
                }
                OpKind::Rand => {
                    if let Some(idx) = op.field_idx {
                        let vals = &agg.string_values[self.string_values_map[&idx]];
                        if vals.is_empty() {
                            values.push("".to_string());
                        } else {
                            let seed = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos()
                                as u64;
                            let mut x = seed;
                            x ^= x << 13;
                            x ^= x >> 7;
                            x ^= x << 17;
                            let index = (x as usize) % vals.len();
                            values.push(vals[index].clone());
                        }
                    }
                }
            }
        }
        values
    }
}

pub struct Aggregator {
    pub count: usize,
    pub sums: Vec<f64>,
    pub sum_sqs: Vec<f64>,  // For variance/stdev
    pub sum_logs: Vec<f64>, // For geomean
    pub sum_invs: Vec<f64>, // For harmmean
    pub mins: Vec<f64>,
    pub maxs: Vec<f64>,
    pub field_counts: Vec<usize>,
    pub values: Vec<Vec<f64>>, // For median/mad/quantiles
    pub firsts: Vec<String>,
    pub lasts: Vec<String>,
    pub value_counts: Vec<HashMap<String, usize>>, // For mode/nunique/unique
    pub string_values: Vec<Vec<String>>,           // For collapse/rand
}

impl Default for Aggregator {
    fn default() -> Self {
        // This Default impl is a bit meaningless without the schema (StatsProcessor),
        // but it satisfies the compiler if needed for placeholders.
        // Real usage should use StatsProcessor::create_aggregator().
        Self {
            count: 0,
            sums: Vec::new(),
            sum_sqs: Vec::new(),
            sum_logs: Vec::new(),
            sum_invs: Vec::new(),
            mins: Vec::new(),
            maxs: Vec::new(),
            field_counts: Vec::new(),
            values: Vec::new(),
            firsts: Vec::new(),
            lasts: Vec::new(),
            value_counts: Vec::new(),
            string_values: Vec::new(),
        }
    }
}

impl Aggregator {
    pub fn calculate_quantile(sorted_vals: &[f64], p: f64) -> f64 {
        let len = sorted_vals.len();
        if len == 0 {
            return f64::NAN;
        }
        if len == 1 {
            return sorted_vals[0];
        }
        let pos = p * (len - 1) as f64;
        let i = pos.floor() as usize;
        let fract = pos - i as f64;
        if i >= len - 1 {
            sorted_vals[len - 1]
        } else {
            sorted_vals[i] * (1.0 - fract) + sorted_vals[i + 1] * fract
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_nan() {
        let ops = vec![Operation {
            kind: OpKind::Mean,
            field_idx: Some(0),
        }];
        let processor = StatsProcessor::new(ops);
        let agg = processor.create_aggregator();
        // Mean needs count > 0 to not be nan
        let results = processor.format_results(&agg);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_mad_nan_no_entry() {
        let ops = vec![Operation {
            kind: OpKind::Mad,
            field_idx: Some(0),
        }];
        let processor = StatsProcessor::new(ops);
        let agg = processor.create_aggregator();
        let results = processor.format_results(&agg);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_stdev_nan() {
        let ops = vec![Operation {
            kind: OpKind::Stdev,
            field_idx: Some(0),
        }];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();
        // Stdev requires count > 1
        // Manually hack state
        let slot = processor.sum_map[&0];
        agg.sums[slot] = 10.0;
        let slot_sq = processor.sum_sq_map[&0];
        agg.sum_sqs[slot_sq] = 100.0;
        let slot_count = processor.count_map[&0];
        agg.field_counts[slot_count] = 1;

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_min_max_nan() {
        let ops = vec![
            Operation {
                kind: OpKind::Min,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Max,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let agg = processor.create_aggregator();
        let results = processor.format_results(&agg);
        assert_eq!(results[0], "nan");
        assert_eq!(results[1], "nan");
    }

    // Helper for testing Row trait
    struct TestRow {
        fields: Vec<String>,
    }
    impl Row for TestRow {
        fn get_bytes(&self, idx: usize) -> Option<&[u8]> {
            if idx == 0 || idx > self.fields.len() {
                None
            } else {
                Some(self.fields[idx - 1].as_bytes())
            }
        }
    }

    #[test]
    fn test_basic_stats() {
        let ops = vec![
            Operation {
                kind: OpKind::Count,
                field_idx: None,
            },
            Operation {
                kind: OpKind::Sum,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Mean,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Min,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Max,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: 10, 20, 30
        let rows = vec![
            TestRow {
                fields: vec!["10".to_string()],
            },
            TestRow {
                fields: vec!["20".to_string()],
            },
            TestRow {
                fields: vec!["30".to_string()],
            },
        ];

        for row in &rows {
            processor.update(&mut agg, row);
        }

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "3"); // Count
        assert_eq!(results[1], "60"); // Sum
        assert_eq!(results[2], "20"); // Mean
        assert_eq!(results[3], "10"); // Min
        assert_eq!(results[4], "30"); // Max
    }

    #[test]
    fn test_variance_stdev_cv() {
        let ops = vec![
            Operation {
                kind: OpKind::Variance,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Stdev,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::CV,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: 2, 4, 4, 4, 5, 5, 7, 9
        // Mean: 5
        // Variance: 4.571428...
        // Stdev: 2.138089...
        // CV: 0.427617...
        let data = vec![2, 4, 4, 4, 5, 5, 7, 9];
        for v in data {
            let row = TestRow {
                fields: vec![v.to_string()],
            };
            processor.update(&mut agg, &row);
        }

        let results = processor.format_results(&agg);
        let var: f64 = results[0].parse().unwrap();
        let stdev: f64 = results[1].parse().unwrap();
        let cv: f64 = results[2].parse().unwrap();

        assert!((var - 4.571428).abs() < 1e-5);
        assert!((stdev - 2.138089).abs() < 1e-5);
        assert!((cv - 0.427617).abs() < 1e-5);
    }

    #[test]
    fn test_quantiles() {
        let ops = vec![
            Operation {
                kind: OpKind::Median,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Q1,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Q3,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::IQR,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: 1, 2, 3, 4, 5
        // Median: 3
        // Q1: 2
        // Q3: 4
        // IQR: 2
        for i in 1..=5 {
            let row = TestRow {
                fields: vec![i.to_string()],
            };
            processor.update(&mut agg, &row);
        }

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "3");
        assert_eq!(results[1], "2");
        assert_eq!(results[2], "4");
        assert_eq!(results[3], "2");
    }

    #[test]
    fn test_geomean_harmmean() {
        let ops = vec![
            Operation {
                kind: OpKind::GeoMean,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::HarmMean,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: 2, 8
        // GeoMean: 4
        // HarmMean: 3.2
        let data = vec![2, 8];
        for v in data {
            let row = TestRow {
                fields: vec![v.to_string()],
            };
            processor.update(&mut agg, &row);
        }

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "4");
        assert_eq!(results[1], "3.2");
    }

    #[test]
    fn test_mode_nunique() {
        let ops = vec![
            Operation {
                kind: OpKind::Mode,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::NUnique,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Unique,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: A, A, B
        let data = vec!["A", "A", "B"];
        for v in data {
            let row = TestRow {
                fields: vec![v.to_string()],
            };
            processor.update(&mut agg, &row);
        }

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "A"); // Mode
        assert_eq!(results[1], "2"); // NUnique
                                     // Unique order is sorted: A,B
        assert_eq!(results[2], "A,B");
    }

    #[test]
    fn test_first_last_range() {
        let ops = vec![
            Operation {
                kind: OpKind::First,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Last,
                field_idx: Some(0),
            },
            Operation {
                kind: OpKind::Range,
                field_idx: Some(0),
            },
        ];
        let processor = StatsProcessor::new(ops);
        let mut agg = processor.create_aggregator();

        // Data: 10, 5, 20
        let data = vec!["10", "5", "20"];
        for v in data {
            let row = TestRow {
                fields: vec![v.to_string()],
            };
            processor.update(&mut agg, &row);
        }

        let results = processor.format_results(&agg);
        assert_eq!(results[0], "10"); // First
        assert_eq!(results[1], "20"); // Last
        assert_eq!(results[2], "15"); // Range (20 - 5)
    }
}
