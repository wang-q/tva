use super::aggregator::Aggregator;
use super::ops::*;
use super::traits::Calculator;
use super::{OpKind, Operation};
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
    pub fn new(ops: Vec<Operation>) -> Self {
        let mut calculators: Vec<Box<dyn Calculator>> = Vec::new();

        // Maps from (field_idx, op_kind_discriminant) to slot_idx
        // We use OpKind as part of the key because different operations on the same field
        // might need different slots (e.g. Sum vs Mean both use sum_slot, but they should be distinct
        // if they are separate operations, OR they should share if we want optimization).
        //
        // However, the original logic seemed to intend sharing slots for the SAME underlying metric.
        // e.g. Mean needs (sum, count). If we also have Sum and Count ops, they should ideally share.
        // But the current implementation of Calculator trait puts the update logic inside each calculator.
        // If multiple calculators share the same slot, they will ALL update it, leading to double counting!
        //
        // EXAMPLE:
        // Ops: [Sum(0), Mean(0)]
        // Sum(0) updates sums[0]
        // Mean(0) updates sums[0] AND counts[0]
        // Result: sums[0] is added TWICE per row!
        //
        // FIX: We must NOT share slots between different calculator instances unless we have a sophisticated
        // dependency graph (which we don't).
        // The simplest fix for correctness is to ALWAYS allocate new slots for each operation.
        //
        // The previous optimization (HashMap lookups) was only valid when we had a single "update" loop
        // that iterated over fields and applied all updates at once.
        // Now that we have decoupled Calculators, each Calculator is an independent entity that drives its own updates.
        // Sharing mutable state (slots) between them without coordination causes the double-update bug.

        // So, we will remove the HashMaps and just increment counters.
        // This might use slightly more memory (e.g. Sum and Mean on same field won't share the sum slot),
        // but it guarantees correctness with the decoupled architecture.
        // Given that number of columns is usually small (< 1000), this memory overhead is negligible compared to
        // the benefits of decoupled code.

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
