use super::math;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Aggregator {
    pub count: usize,
    pub sums: Vec<f64>,
    pub sum_sqs: Vec<f64>,  // For variance/stdev
    pub sum_logs: Vec<f64>, // For geomean
    pub sum_invs: Vec<f64>, // For harmmean
    pub mins: Vec<f64>,
    pub maxs: Vec<f64>,
    pub field_counts: Vec<usize>,
    pub values: Vec<Vec<f64>>, // For median/mad/quantiles
    pub firsts: Vec<String>,
    pub lasts: Vec<String>,
    pub value_counts: Vec<HashMap<String, usize>>, // For mode/nunique/unique
    pub string_values: Vec<Vec<String>>,           // For collapse/rand
}

impl Default for Aggregator {
    fn default() -> Self {
        // This Default impl is a bit meaningless without the schema (StatsProcessor),
        // but it satisfies the compiler if needed for placeholders.
        // Real usage should use StatsProcessor::create_aggregator().
        Self {
            count: 0,
            sums: Vec::new(),
            sum_sqs: Vec::new(),
            sum_logs: Vec::new(),
            sum_invs: Vec::new(),
            mins: Vec::new(),
            maxs: Vec::new(),
            field_counts: Vec::new(),
            values: Vec::new(),
            firsts: Vec::new(),
            lasts: Vec::new(),
            value_counts: Vec::new(),
            string_values: Vec::new(),
        }
    }
}

impl Aggregator {
    /// Calculate quantile on a SORTED slice.
    /// This is a convenience wrapper around math::quantile.
    #[inline]
    pub fn calculate_quantile(sorted_vals: &[f64], p: f64) -> f64 {
        math::quantile(sorted_vals, p)
    }
}
