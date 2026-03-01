use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::tsv::record::Row;

pub struct Median {
    pub field_idx: usize,
    pub values_slot: usize,
}

impl Calculator for Median {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.5);
            if res.is_nan() {
                "nan".to_string()
            } else {
                res.to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}

pub struct Q1 {
    pub field_idx: usize,
    pub values_slot: usize,
}

impl Calculator for Q1 {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.25);
            if res.is_nan() {
                "nan".to_string()
            } else {
                res.to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}

pub struct Q3 {
    pub field_idx: usize,
    pub values_slot: usize,
}

impl Calculator for Q3 {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.75);
            if res.is_nan() {
                "nan".to_string()
            } else {
                res.to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}

pub struct IQR {
    pub field_idx: usize,
    pub values_slot: usize,
}

impl Calculator for IQR {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q1 = math::quantile(&sorted_vals, 0.25);
            let q3 = math::quantile(&sorted_vals, 0.75);
            let res = q3 - q1;
            if res.is_nan() {
                "nan".to_string()
            } else {
                res.to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}

pub struct Mad {
    pub field_idx: usize,
    pub values_slot: usize,
}

impl Calculator for Mad {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) = parse_float(row, self.field_idx) {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::mad(&sorted_vals);
            if res.is_nan() {
                "nan".to_string()
            } else {
                res.to_string()
            }
        } else {
            "nan".to_string()
        }
    }
}
