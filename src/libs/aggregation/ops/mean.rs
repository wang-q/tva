use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Mean {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Mean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum = agg.sums[self.sum_slot];
        let res = math::mean(sum, count);
        format_float(res, self.precision)
    }
}

pub struct GeoMean {
    pub field_idx: usize,
    pub sum_log_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for GeoMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val > 0.0 {
                agg.sum_logs[self.sum_log_slot] += val.ln();
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum_log = agg.sum_logs[self.sum_log_slot];
        let res = math::geomean(sum_log, count);
        format_float(res, self.precision)
    }
}

pub struct HarmMean {
    pub field_idx: usize,
    pub sum_inv_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for HarmMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val != 0.0 {
                agg.sum_invs[self.sum_inv_slot] += 1.0 / val;
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum_inv = agg.sum_invs[self.sum_inv_slot];
        let res = math::harmmean(sum_inv, count);
        format_float(res, self.precision)
    }
}
