use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::tsv::record::Row;

pub struct Count;

impl Calculator for Count {
    fn update(&self, agg: &mut Aggregator, _row: &dyn Row) {
        agg.count += 1;
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.count.to_string()
    }
}

pub struct Sum {
    pub field_idx: usize,
    pub sum_slot: usize,
}

impl Calculator for Sum {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.sums[self.sum_slot] += val;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.sums[self.sum_slot].to_string()
    }
}

pub struct Min {
    pub field_idx: usize,
    pub min_slot: usize,
}

impl Calculator for Min {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            if val < agg.mins[self.min_slot] {
                agg.mins[self.min_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let val = agg.mins[self.min_slot];
        if val == f64::INFINITY {
            "nan".to_string()
        } else {
            val.to_string()
        }
    }
}

pub struct Max {
    pub field_idx: usize,
    pub max_slot: usize,
}

impl Calculator for Max {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            if val > agg.maxs[self.max_slot] {
                agg.maxs[self.max_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let val = agg.maxs[self.max_slot];
        if val == f64::NEG_INFINITY {
            "nan".to_string()
        } else {
            val.to_string()
        }
    }
}

pub struct Range {
    pub field_idx: usize,
    pub min_slot: usize,
    pub max_slot: usize,
}

impl Calculator for Range {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            if val < agg.mins[self.min_slot] {
                agg.mins[self.min_slot] = val;
            }
            if val > agg.maxs[self.max_slot] {
                agg.maxs[self.max_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let min = agg.mins[self.min_slot];
        let max = agg.maxs[self.max_slot];
        if min != f64::INFINITY && max != f64::NEG_INFINITY {
            (max - min).to_string()
        } else {
            "nan".to_string()
        }
    }
}
