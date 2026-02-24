use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
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
}

pub struct Operation {
    pub kind: OpKind,
    pub field_idx: Option<usize>, // None for count
}

pub struct Aggregator {
    pub count: usize,
    pub sums: HashMap<usize, f64>,
    pub sum_sqs: HashMap<usize, f64>, // For variance/stdev
    pub sum_logs: HashMap<usize, f64>, // For geomean
    pub sum_invs: HashMap<usize, f64>, // For harmmean
    pub mins: HashMap<usize, f64>,
    pub maxs: HashMap<usize, f64>,
    pub field_counts: HashMap<usize, usize>,
    pub values: HashMap<usize, Vec<f64>>, // For median/mad/quantiles
    pub firsts: HashMap<usize, String>,
    pub lasts: HashMap<usize, String>,
    pub value_counts: HashMap<usize, HashMap<String, usize>>, // For mode/nunique
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            count: 0,
            sums: HashMap::new(),
            sum_sqs: HashMap::new(),
            sum_logs: HashMap::new(),
            sum_invs: HashMap::new(),
            mins: HashMap::new(),
            maxs: HashMap::new(),
            field_counts: HashMap::new(),
            values: HashMap::new(),
            firsts: HashMap::new(),
            lasts: HashMap::new(),
            value_counts: HashMap::new(),
        }
    }

    pub fn update(&mut self, record: &[&[u8]], ops: &[Operation]) {
        self.count += 1;

        // Collect fields needed for each type of operation
        let mut sum_fields = Vec::new();
        let mut sum_sq_fields = Vec::new();
        let mut sum_log_fields = Vec::new();
        let mut sum_inv_fields = Vec::new();
        let mut value_fields = Vec::new();
        let mut first_fields = Vec::new();
        let mut last_fields = Vec::new();
        let mut count_fields = Vec::new();

        for op in ops {
            if let Some(idx) = op.field_idx {
                match op.kind {
                    OpKind::Sum | OpKind::Mean => sum_fields.push(idx),
                    OpKind::Stdev | OpKind::Variance | OpKind::CV => {
                        sum_fields.push(idx);
                        sum_sq_fields.push(idx);
                    }
                    OpKind::GeoMean => sum_log_fields.push(idx),
                    OpKind::HarmMean => sum_inv_fields.push(idx),
                    OpKind::Median | OpKind::Mad | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
                        value_fields.push(idx)
                    }
                    OpKind::First => first_fields.push(idx),
                    OpKind::Last => last_fields.push(idx),
                    OpKind::NUnique | OpKind::Mode => count_fields.push(idx),
                    OpKind::Range => { /* Handled in Min/Max logic block */ }
                    _ => {}
                }
            }
        }

        sum_fields.sort_unstable();
        sum_fields.dedup();
        sum_sq_fields.sort_unstable();
        sum_sq_fields.dedup();
        sum_log_fields.sort_unstable();
        sum_log_fields.dedup();
        sum_inv_fields.sort_unstable();
        sum_inv_fields.dedup();
        value_fields.sort_unstable();
        value_fields.dedup();
        first_fields.sort_unstable();
        first_fields.dedup();
        last_fields.sort_unstable();
        last_fields.dedup();
        count_fields.sort_unstable();
        count_fields.dedup();

        // Handle Sum/Mean/Stdev/Variance/CV
        for idx in &sum_fields {
            if *idx >= record.len() {
                continue;
            }
            let val_bytes = record[*idx];
            if val_bytes.is_empty() {
                continue;
            }
            if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                if let Ok(val) = val_str.trim().parse::<f64>() {
                    *self.sums.entry(*idx).or_insert(0.0) += val;
                    *self.field_counts.entry(*idx).or_insert(0) += 1;

                    if sum_sq_fields.contains(idx) {
                        *self.sum_sqs.entry(*idx).or_insert(0.0) += val * val;
                    }
                }
            }
        }

        // Handle GeoMean
        for idx in &sum_log_fields {
            if *idx >= record.len() {
                continue;
            }
            let val_bytes = record[*idx];
            if val_bytes.is_empty() {
                continue;
            }
            if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                if let Ok(val) = val_str.trim().parse::<f64>() {
                    if val > 0.0 {
                        *self.sum_logs.entry(*idx).or_insert(0.0) += val.ln();
                        // If field_counts not updated by sum_fields, update here?
                        // Assuming user might ask for geomean only.
                        if !sum_fields.contains(idx) {
                            *self.field_counts.entry(*idx).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Handle HarmMean
        for idx in &sum_inv_fields {
            if *idx >= record.len() {
                continue;
            }
            let val_bytes = record[*idx];
            if val_bytes.is_empty() {
                continue;
            }
            if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                if let Ok(val) = val_str.trim().parse::<f64>() {
                    if val != 0.0 {
                        *self.sum_invs.entry(*idx).or_insert(0.0) += 1.0 / val;
                        if !sum_fields.contains(idx) && !sum_log_fields.contains(idx) {
                            *self.field_counts.entry(*idx).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Handle Median/Mad/Quantiles (store all values)

        for idx in value_fields {
            if idx >= record.len() {
                continue;
            }
            let val_bytes = record[idx];
            if val_bytes.is_empty() {
                continue;
            }
            if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                if let Ok(val) = val_str.trim().parse::<f64>() {
                    self.values.entry(idx).or_default().push(val);
                }
            }
        }

        // Handle First
        for idx in first_fields {
            if !self.firsts.contains_key(&idx) {
                if idx < record.len() {
                    let val = String::from_utf8_lossy(record[idx]).to_string();
                    self.firsts.insert(idx, val);
                } else {
                    self.firsts.insert(idx, String::new());
                }
            }
        }

        // Handle Last
        for idx in last_fields {
            if idx < record.len() {
                let val = String::from_utf8_lossy(record[idx]).to_string();
                self.lasts.insert(idx, val);
            } else {
                self.lasts.insert(idx, String::new());
            }
        }

        // Handle NUnique/Mode
        for idx in count_fields {
            if idx < record.len() {
                let val = String::from_utf8_lossy(record[idx]).to_string();
                *self
                    .value_counts
                    .entry(idx)
                    .or_default()
                    .entry(val)
                    .or_insert(0) += 1;
            }
        }

        // Handle Min/Max
        for op in ops {
            if let Some(idx) = op.field_idx {
                if idx >= record.len() {
                    continue;
                }
                let val_bytes = record[idx];
                if val_bytes.is_empty() {
                    continue;
                }

                if matches!(op.kind, OpKind::Min | OpKind::Max | OpKind::Range) {
                    if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                        if let Ok(val) = val_str.trim().parse::<f64>() {
                            match op.kind {
                                OpKind::Min | OpKind::Range => {
                                    let entry =
                                        self.mins.entry(idx).or_insert(f64::INFINITY);
                                    if val < *entry {
                                        *entry = val;
                                    }
                                }
                                _ => {}
                            }
                            match op.kind {
                                OpKind::Max | OpKind::Range => {
                                    let entry = self
                                        .maxs
                                        .entry(idx)
                                        .or_insert(f64::NEG_INFINITY);
                                    if val > *entry {
                                        *entry = val;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

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

    pub fn format_results(&self, ops: &[Operation]) -> Vec<String> {
        let mut values = Vec::new();
        for op in ops {
            match op.kind {
                OpKind::Count => values.push(self.count.to_string()),
                OpKind::Sum => {
                    if let Some(idx) = op.field_idx {
                        let val = self.sums.get(&idx).copied().unwrap_or(0.0);
                        values.push(val.to_string());
                    }
                }
                OpKind::Mean => {
                    if let Some(idx) = op.field_idx {
                        let sum = self.sums.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);
                        if count > 0 {
                            values.push((sum / count as f64).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Min => {
                    if let Some(idx) = op.field_idx {
                        let val = self.mins.get(&idx).copied().unwrap_or(f64::INFINITY);
                        if val == f64::INFINITY {
                            values.push("nan".to_string());
                        } else {
                            values.push(val.to_string());
                        }
                    }
                }
                OpKind::Max => {
                    if let Some(idx) = op.field_idx {
                        let val =
                            self.maxs.get(&idx).copied().unwrap_or(f64::NEG_INFINITY);
                        if val == f64::NEG_INFINITY {
                            values.push("nan".to_string());
                        } else {
                            values.push(val.to_string());
                        }
                    }
                }
                OpKind::GeoMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_log = self.sum_logs.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);
                        if count > 0 {
                            values.push((sum_log / count as f64).exp().to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::HarmMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_inv = self.sum_invs.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);
                        if count > 0 && sum_inv != 0.0 {
                            values.push((count as f64 / sum_inv).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Range => {
                    if let Some(idx) = op.field_idx {
                        let min = self.mins.get(&idx).copied().unwrap_or(f64::INFINITY);
                        let max = self.maxs.get(&idx).copied().unwrap_or(f64::NEG_INFINITY);
                        if min != f64::INFINITY && max != f64::NEG_INFINITY {
                            values.push((max - min).to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::CV => {
                    if let Some(idx) = op.field_idx {
                        let sum = self.sums.get(&idx).copied().unwrap_or(0.0);
                        let sum_sq = self.sum_sqs.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);

                        if count > 1 {
                            let mean = sum / count as f64;
                            let variance = (sum_sq - (sum * sum) / count as f64)
                                / (count as f64 - 1.0);
                            let stdev = variance.sqrt();
                            if mean != 0.0 {
                                values.push((stdev / mean).to_string());
                            } else {
                                values.push("nan".to_string());
                            }
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Median | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
                    if let Some(idx) = op.field_idx {
                        if let Some(vals) = self.values.get(&idx) {
                            let mut sorted_vals = vals.clone();
                            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let len = sorted_vals.len();

                            if len > 0 {
                                match op.kind {
                                    OpKind::Median => values.push(Self::calculate_quantile(&sorted_vals, 0.5).to_string()),
                                    OpKind::Q1 => values.push(Self::calculate_quantile(&sorted_vals, 0.25).to_string()),
                                    OpKind::Q3 => values.push(Self::calculate_quantile(&sorted_vals, 0.75).to_string()),
                                    OpKind::IQR => {
                                        let q1 = Self::calculate_quantile(&sorted_vals, 0.25);
                                        let q3 = Self::calculate_quantile(&sorted_vals, 0.75);
                                        values.push((q3 - q1).to_string());
                                    }
                                    _ => {}
                                }
                            } else {
                                values.push("nan".to_string());
                            }
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Stdev => {
                    if let Some(idx) = op.field_idx {
                        let sum = self.sums.get(&idx).copied().unwrap_or(0.0);
                        let sum_sq = self.sum_sqs.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);

                        if count > 1 {
                            let variance = (sum_sq - (sum * sum) / count as f64)
                                / (count as f64 - 1.0);
                            values.push(variance.sqrt().to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Variance => {
                    if let Some(idx) = op.field_idx {
                        let sum = self.sums.get(&idx).copied().unwrap_or(0.0);
                        let sum_sq = self.sum_sqs.get(&idx).copied().unwrap_or(0.0);
                        let count = self.field_counts.get(&idx).copied().unwrap_or(0);

                        if count > 1 {
                            let variance = (sum_sq - (sum * sum) / count as f64)
                                / (count as f64 - 1.0);
                            values.push(variance.to_string());
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::Mad => {
                    if let Some(idx) = op.field_idx {
                        if let Some(vals) = self.values.get(&idx) {
                            let mut sorted_vals = vals.clone();
                            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let len = sorted_vals.len();
                            if len > 0 {
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
                        } else {
                            values.push("nan".to_string());
                        }
                    }
                }
                OpKind::First => {
                    if let Some(idx) = op.field_idx {
                        values.push(self.firsts.get(&idx).cloned().unwrap_or_default());
                    }
                }
                OpKind::Last => {
                    if let Some(idx) = op.field_idx {
                        values.push(self.lasts.get(&idx).cloned().unwrap_or_default());
                    }
                }
                OpKind::NUnique => {
                    if let Some(idx) = op.field_idx {
                        if let Some(counts) = self.value_counts.get(&idx) {
                            values.push(counts.len().to_string());
                        } else {
                            values.push("0".to_string());
                        }
                    }
                }
                OpKind::Mode => {
                    if let Some(idx) = op.field_idx {
                        if let Some(counts) = self.value_counts.get(&idx) {
                            if counts.is_empty() {
                                values.push("".to_string());
                            } else {
                                // Sort by count desc, then by value asc
                                let mut count_vec: Vec<(&String, &usize)> =
                                    counts.iter().collect();
                                count_vec.sort_by(|a, b| {
                                    b.1.cmp(a.1).then_with(|| a.0.cmp(b.0))
                                });
                                values.push(count_vec[0].0.clone());
                            }
                        } else {
                            values.push("".to_string());
                        }
                    }
                }
            }
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_nan() {
        let agg = Aggregator::new();
        // Mean needs count > 0 to not be nan
        let ops = vec![Operation {
            kind: OpKind::Mean,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_mad_nan_no_entry() {
        let agg = Aggregator::new();
        let ops = vec![Operation {
            kind: OpKind::Mad,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_mad_nan_empty_vec() {
        let mut agg = Aggregator::new();
        // Manually insert empty vector to trigger the specific branch (L331-332)
        agg.values.insert(0, vec![]);

        let ops = vec![Operation {
            kind: OpKind::Mad,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_median_nan_no_entry() {
        let agg = Aggregator::new();
        let ops = vec![Operation {
            kind: OpKind::Median,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_median_nan_empty_vec() {
        let mut agg = Aggregator::new();
        // Manually insert empty vector to trigger the specific branch (L267-268)
        agg.values.insert(0, vec![]);

        let ops = vec![Operation {
            kind: OpKind::Median,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_stdev_nan() {
        let mut agg = Aggregator::new();
        // Stdev requires count > 1
        agg.field_counts.insert(0, 1);
        agg.sums.insert(0, 10.0);
        agg.sum_sqs.insert(0, 100.0);

        let ops = vec![Operation {
            kind: OpKind::Stdev,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_variance_nan() {
        let mut agg = Aggregator::new();
        // Variance requires count > 1
        agg.field_counts.insert(0, 1);

        let ops = vec![Operation {
            kind: OpKind::Variance,
            field_idx: Some(0),
        }];
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
    }

    #[test]
    fn test_min_max_nan() {
        let agg = Aggregator::new();

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
        // format_results processes ops in order
        let results = agg.format_results(&ops);
        assert_eq!(results[0], "nan");
        assert_eq!(results[1], "nan");
    }
}
