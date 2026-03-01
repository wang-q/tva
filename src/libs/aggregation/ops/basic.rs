use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Count;

impl Calculator for Count {
    fn update(&self, agg: &mut Aggregator, _row: &dyn Row) {
        agg.count += 1;
    }

    fn format(&self, _agg: &Aggregator) -> String {
        _agg.count.to_string()
    }
}

pub struct Sum {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Sum {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx, self.missing_val, self.exclude_missing) {
            agg.sums[self.sum_slot] += val;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        format_float(agg.sums[self.sum_slot], self.precision)
    }
}

pub struct Min {
    pub field_idx: usize,
    pub min_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Min {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx, self.missing_val, self.exclude_missing) {
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
            format_float(val, self.precision)
        }
    }
}

pub struct Max {
    pub field_idx: usize,
    pub max_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Max {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx, self.missing_val, self.exclude_missing) {
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
            format_float(val, self.precision)
        }
    }
}

pub struct Range {
    pub field_idx: usize,
    pub min_slot: usize,
    pub max_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Range {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx, self.missing_val, self.exclude_missing) {
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
        let res = math::range(min, max);
        format_float(res, self.precision)
    }
}

pub struct MissingCount {
    pub field_idx: usize,
    pub count_slot: usize,
}

impl Calculator for MissingCount {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let is_missing = match row.get_bytes(self.field_idx + 1) {
            None => true,
            Some(bytes) => bytes.is_empty(),
        };
        if is_missing {
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.field_counts[self.count_slot].to_string()
    }
}

pub struct NotMissingCount {
    pub field_idx: usize,
    pub count_slot: usize,
}

impl Calculator for NotMissingCount {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let is_missing = match row.get_bytes(self.field_idx + 1) {
            None => true,
            Some(bytes) => bytes.is_empty(),
        };
        if !is_missing {
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.field_counts[self.count_slot].to_string()
    }
}
