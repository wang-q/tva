use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::aggregation::ops::parse_float;
use crate::libs::tsv::record::Row;

pub struct Mean {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub count_slot: usize,
}

impl Calculator for Mean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.sums[self.sum_slot] += val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        if count > 0 {
            (agg.sums[self.sum_slot] / count as f64).to_string()
        } else {
            "nan".to_string()
        }
    }
}

pub struct GeoMean {
    pub field_idx: usize,
    pub sum_log_slot: usize,
    pub count_slot: usize,
}

impl Calculator for GeoMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            if val > 0.0 {
                agg.sum_logs[self.sum_log_slot] += val.ln();
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        if count > 0 {
            (agg.sum_logs[self.sum_log_slot] / count as f64).exp().to_string()
        } else {
            "nan".to_string()
        }
    }
}

pub struct HarmMean {
    pub field_idx: usize,
    pub sum_inv_slot: usize,
    pub count_slot: usize,
}

impl Calculator for HarmMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            if val != 0.0 {
                agg.sum_invs[self.sum_inv_slot] += 1.0 / val;
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum_inv = agg.sum_invs[self.sum_inv_slot];
        if count > 0 && sum_inv != 0.0 {
            (count as f64 / sum_inv).to_string()
        } else {
            "nan".to_string()
        }
    }
}
