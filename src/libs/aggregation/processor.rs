use super::aggregator::Aggregator;
use super::ops::*;
use super::traits::Calculator;
use super::{OpKind, Operation, StatsConfig};
use crate::libs::tsv::record::Row;
use std::collections::HashMap;

/// Processor that manages the schema and memory layout for aggregations
pub struct StatsProcessor {
    pub calculators: Vec<Box<dyn Calculator>>,
    // Metadata for slots allocation
    num_sums: usize,
    num_sum_sqs: usize,
    num_sum_logs: usize,
    num_sum_invs: usize,
    num_mins: usize,
    num_maxs: usize,
    num_counts: usize,
    num_values: usize,
    num_firsts: usize,
    num_lasts: usize,
    num_value_counts: usize,
    num_string_values: usize,
}

impl StatsProcessor {
    pub fn new(ops: Vec<Operation>, config: StatsConfig) -> Self {
        let mut calculators: Vec<Box<dyn Calculator>> = Vec::new();

        // Maps from (field_idx, op_kind_discriminant) to slot_idx
        // We do NOT share slots between operations (e.g. Sum and Mean on the same field)
        // to avoid double-counting when multiple calculators update the same slot independently.
        // Each calculator gets its own dedicated slot(s).

        let mut num_sums = 0;
        let mut num_sum_sqs = 0;
        let mut num_sum_logs = 0;
        let mut num_sum_invs = 0;
        let mut num_mins = 0;
        let mut num_maxs = 0;
        let mut num_counts = 0;
        let mut num_values = 0;
        let mut num_firsts = 0;
        let mut num_lasts = 0;
        let mut num_value_counts = 0;
        let mut num_string_values = 0;

        for op in ops {
            match op.kind {
                OpKind::Count => {
                    calculators.push(Box::new(basic::Count));
                }
                OpKind::Sum => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_sums;
                        num_sums += 1;
                        calculators.push(Box::new(basic::Sum {
                            field_idx: idx,
                            sum_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Min => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_mins;
                        num_mins += 1;
                        calculators.push(Box::new(basic::Min {
                            field_idx: idx,
                            min_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Max => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_maxs;
                        num_maxs += 1;
                        calculators.push(Box::new(basic::Max {
                            field_idx: idx,
                            max_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Range => {
                    if let Some(idx) = op.field_idx {
                        let min_slot = num_mins;
                        num_mins += 1;
                        let max_slot = num_maxs;
                        num_maxs += 1;
                        calculators.push(Box::new(basic::Range {
                            field_idx: idx,
                            min_slot,
                            max_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Mean => {
                    if let Some(idx) = op.field_idx {
                        let sum_slot = num_sums;
                        num_sums += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(mean::Mean {
                            field_idx: idx,
                            sum_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::GeoMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_log_slot = num_sum_logs;
                        num_sum_logs += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(mean::GeoMean {
                            field_idx: idx,
                            sum_log_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::HarmMean => {
                    if let Some(idx) = op.field_idx {
                        let sum_inv_slot = num_sum_invs;
                        num_sum_invs += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(mean::HarmMean {
                            field_idx: idx,
                            sum_inv_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Variance => {
                    if let Some(idx) = op.field_idx {
                        let sum_slot = num_sums;
                        num_sums += 1;
                        let sum_sq_slot = num_sum_sqs;
                        num_sum_sqs += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(variance::Variance {
                            field_idx: idx,
                            sum_slot,
                            sum_sq_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Stdev => {
                    if let Some(idx) = op.field_idx {
                        let sum_slot = num_sums;
                        num_sums += 1;
                        let sum_sq_slot = num_sum_sqs;
                        num_sum_sqs += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(variance::Stdev {
                            field_idx: idx,
                            sum_slot,
                            sum_sq_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::CV => {
                    if let Some(idx) = op.field_idx {
                        let sum_slot = num_sums;
                        num_sums += 1;
                        let sum_sq_slot = num_sum_sqs;
                        num_sum_sqs += 1;
                        let count_slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(variance::CV {
                            field_idx: idx,
                            sum_slot,
                            sum_sq_slot,
                            count_slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Median => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::Median {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Q1 => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::Q1 {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Q3 => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::Q3 {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::IQR => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::IQR {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::Mad => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::Mad {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
                OpKind::First => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_firsts;
                        num_firsts += 1;
                        calculators.push(Box::new(text::First {
                            field_idx: idx,
                            first_slot: slot,
                        }));
                    }
                }
                OpKind::Last => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_lasts;
                        num_lasts += 1;
                        calculators.push(Box::new(text::Last {
                            field_idx: idx,
                            last_slot: slot,
                        }));
                    }
                }
                OpKind::NUnique => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_value_counts;
                        num_value_counts += 1;
                        calculators.push(Box::new(set::NUnique {
                            field_idx: idx,
                            value_counts_slot: slot,
                        }));
                    }
                }
                OpKind::Mode => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_value_counts;
                        num_value_counts += 1;
                        calculators.push(Box::new(set::Mode {
                            field_idx: idx,
                            value_counts_slot: slot,
                        }));
                    }
                }
                OpKind::Unique => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_value_counts;
                        num_value_counts += 1;
                        calculators.push(Box::new(set::Unique {
                            field_idx: idx,
                            value_counts_slot: slot,
                            delimiter: config.delimiter.to_string(),
                        }));
                    }
                }
                OpKind::Collapse => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_string_values;
                        num_string_values += 1;
                        calculators.push(Box::new(text::Collapse {
                            field_idx: idx,
                            string_values_slot: slot,
                            delimiter: config.delimiter.to_string(),
                        }));
                    }
                }
                OpKind::Rand => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_string_values;
                        num_string_values += 1;
                        calculators.push(Box::new(text::Rand {
                            field_idx: idx,
                            string_values_slot: slot,
                        }));
                    }
                }
                OpKind::ModeCount => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_value_counts;
                        num_value_counts += 1;
                        calculators.push(Box::new(set::ModeCount {
                            field_idx: idx,
                            value_counts_slot: slot,
                        }));
                    }
                }
                OpKind::MissingCount => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(basic::MissingCount {
                            field_idx: idx,
                            count_slot: slot,
                        }));
                    }
                }
                OpKind::NotMissingCount => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_counts;
                        num_counts += 1;
                        calculators.push(Box::new(basic::NotMissingCount {
                            field_idx: idx,
                            count_slot: slot,
                        }));
                    }
                }
                OpKind::Quantile(p) => {
                    if let Some(idx) = op.field_idx {
                        let slot = num_values;
                        num_values += 1;
                        calculators.push(Box::new(quantile::Quantile {
                            field_idx: idx,
                            values_slot: slot,
                            precision: config.precision,
                            probability: p,
                            missing_val: config.missing_val_f64,
                            exclude_missing: config.exclude_missing,
                        }));
                    }
                }
            }
        }

        Self {
            calculators,
            num_sums,
            num_sum_sqs,
            num_sum_logs,
            num_sum_invs,
            num_mins,
            num_maxs,
            num_counts,
            num_values,
            num_firsts,
            num_lasts,
            num_value_counts,
            num_string_values,
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
            field_counts: vec![0; self.num_counts],
            values: vec![Vec::new(); self.num_values],
            firsts: vec![String::new(); self.num_firsts],
            lasts: vec![String::new(); self.num_lasts],
            value_counts: vec![HashMap::new(); self.num_value_counts],
            string_values: vec![Vec::new(); self.num_string_values],
        }
    }

    #[inline]
    pub fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        for calc in &self.calculators {
            calc.update(agg, row);
        }
    }

    pub fn format_results(&self, agg: &Aggregator) -> Vec<String> {
        self.calculators.iter().map(|c| c.format(agg)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::aggregation::{OpKind, Operation, StatsConfig};
    use crate::libs::tsv::record::Row;

    #[test]
    fn test_processor_allocation() {
        let ops = vec![
            Operation {
                kind: OpKind::Count,
                field_idx: None,
            }, // No slots
            Operation {
                kind: OpKind::Sum,
                field_idx: Some(1),
            }, // sum: 1
            Operation {
                kind: OpKind::Mean,
                field_idx: Some(2),
            }, // sum: 2, count: 1
            Operation {
                kind: OpKind::Variance,
                field_idx: Some(3),
            }, // sum: 3, sum_sq: 1, count: 2
            Operation {
                kind: OpKind::Min,
                field_idx: Some(4),
            }, // min: 1
            Operation {
                kind: OpKind::Max,
                field_idx: Some(4),
            }, // max: 1
            Operation {
                kind: OpKind::Range,
                field_idx: Some(5),
            }, // min: 2, max: 2
        ];

        let config = StatsConfig::default();
        let processor = StatsProcessor::new(ops, config);

        assert_eq!(processor.num_sums, 3); // Sum(1) + Mean(1) + Variance(1)
        assert_eq!(processor.num_counts, 2); // Mean(1) + Variance(1)
        assert_eq!(processor.num_sum_sqs, 1); // Variance(1)
        assert_eq!(processor.num_mins, 2); // Min(1) + Range(1)
        assert_eq!(processor.num_maxs, 2); // Max(1) + Range(1)

        let agg = processor.create_aggregator();
        assert_eq!(agg.sums.len(), 3);
        assert_eq!(agg.field_counts.len(), 2);
        assert_eq!(agg.sum_sqs.len(), 1);
        assert_eq!(agg.mins.len(), 2);
        assert_eq!(agg.maxs.len(), 2);
    }

    #[test]
    fn test_processor_update() {
        let ops = vec![Operation {
            kind: OpKind::Sum,
            field_idx: Some(0),
        }];
        let processor = StatsProcessor::new(ops, StatsConfig::default());
        let mut agg = processor.create_aggregator();

        // Mock Row
        struct TestRow(Vec<String>);
        impl Row for TestRow {
            fn get_bytes(&self, index: usize) -> Option<&[u8]> {
                if index == 0 || index > self.0.len() {
                    None
                } else {
                    Some(self.0[index - 1].as_bytes())
                }
            }
        }

        let row1 = TestRow(vec!["10.0".to_string()]);
        let row2 = TestRow(vec!["20.0".to_string()]);

        processor.update(&mut agg, &row1);
        processor.update(&mut agg, &row2);

        let results = processor.format_results(&agg);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "30"); // 10 + 20
    }

    #[test]
    fn test_processor_more_ops() {
        let ops = vec![
            Operation {
                kind: OpKind::GeoMean,
                field_idx: Some(1),
            }, // sum_logs, count
            Operation {
                kind: OpKind::HarmMean,
                field_idx: Some(1),
            }, // sum_invs, count
            Operation {
                kind: OpKind::Median,
                field_idx: Some(1),
            }, // values
            Operation {
                kind: OpKind::First,
                field_idx: Some(1),
            }, // firsts
            Operation {
                kind: OpKind::Last,
                field_idx: Some(1),
            }, // lasts
            Operation {
                kind: OpKind::Unique,
                field_idx: Some(1),
            }, // value_counts
            Operation {
                kind: OpKind::Collapse,
                field_idx: Some(1),
            }, // string_values
        ];
        let processor = StatsProcessor::new(ops, StatsConfig::default());

        assert_eq!(processor.num_sum_logs, 1);
        assert_eq!(processor.num_sum_invs, 1);
        assert_eq!(processor.num_counts, 2); // GeoMean + HarmMean
        assert_eq!(processor.num_values, 1);
        assert_eq!(processor.num_firsts, 1);
        assert_eq!(processor.num_lasts, 1);
        assert_eq!(processor.num_value_counts, 1);
        assert_eq!(processor.num_string_values, 1);
    }
}
