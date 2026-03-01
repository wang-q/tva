use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::aggregation::ops::parse_float;
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
            Aggregator::calculate_quantile(&sorted_vals, 0.5).to_string()
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
            Aggregator::calculate_quantile(&sorted_vals, 0.25).to_string()
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
            Aggregator::calculate_quantile(&sorted_vals, 0.75).to_string()
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
            let q1 = Aggregator::calculate_quantile(&sorted_vals, 0.25);
            let q3 = Aggregator::calculate_quantile(&sorted_vals, 0.75);
            (q3 - q1).to_string()
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
            let median = Aggregator::calculate_quantile(&sorted_vals, 0.5);
            
            let mut deviations: Vec<f64> = vals
                .iter()
                .map(|v| (v - median).abs())
                .collect();
            deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let mad = Aggregator::calculate_quantile(&deviations, 0.5);
            
            // Scale by 1.4826 to be consistent with normal distribution (like R's mad)
            (mad * 1.4826).to_string()
        } else {
            "nan".to_string()
        }
    }
}
