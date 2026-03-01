use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::aggregation::ops::parse_float;
use crate::libs::tsv::record::Row;

pub struct Variance {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
}

impl Calculator for Variance {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        if count > 1 {
            let sum = agg.sums[self.sum_slot];
            let sum_sq = agg.sum_sqs[self.sum_sq_slot];
            let variance = (sum_sq - (sum * sum) / count as f64) / (count as f64 - 1.0);
            variance.to_string()
        } else {
            "nan".to_string()
        }
    }
}

pub struct Stdev {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
}

impl Calculator for Stdev {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        if count > 1 {
            let sum = agg.sums[self.sum_slot];
            let sum_sq = agg.sum_sqs[self.sum_sq_slot];
            let variance = (sum_sq - (sum * sum) / count as f64) / (count as f64 - 1.0);
            variance.sqrt().to_string()
        } else {
            "nan".to_string()
        }
    }
}

pub struct CV {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
}

impl Calculator for CV {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        if count > 1 {
            let sum = agg.sums[self.sum_slot];
            let sum_sq = agg.sum_sqs[self.sum_sq_slot];
            let variance = (sum_sq - (sum * sum) / count as f64) / (count as f64 - 1.0);
            let mean = sum / count as f64;
            if mean != 0.0 {
                (variance.sqrt() / mean).to_string()
            } else {
                "nan".to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}
