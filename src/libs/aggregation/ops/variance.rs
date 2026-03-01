use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::math;
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
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::variance(sum_sq, sum, count);
        if res.is_nan() { "nan".to_string() } else { res.to_string() }
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
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::stdev(sum_sq, sum, count);
        if res.is_nan() { "nan".to_string() } else { res.to_string() }
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
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::cv(sum_sq, sum, count);
        if res.is_nan() { "nan".to_string() } else { res.to_string() }
    }
}
