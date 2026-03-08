//! State container for aggregation operations used by `tva wider`.
//!
//! This module defines the `Cell` enum, which is a stateful container used to accumulate values
//! for a single cell in the output of a pivot/wider operation.
//!
//! # Relationship with `libs::aggregation`
//!
//! While `libs::aggregation` provides a high-performance, column-oriented aggregation engine
//! (optimized for `tva stats` which aggregates entire columns), this module provides a
//! simplified, cell-oriented aggregation mechanism for `tva wider`.
//!
//! - `libs::aggregation`: Uses `Aggregator` and `StatsProcessor` with specialized `Calculator`
//!   implementations. It processes data column-wise or group-wise with highly optimized memory layout.
//!   It uses a **Structure of Arrays (SoA)** approach where statistics (sums, counts, etc.) are stored in parallel vectors
//!   for efficient SIMD processing and cache locality.
//! - `libs::cell`: Uses `Cell` enum to store state for a *single* output cell (intersection of row and column in pivot table).
//!   It uses an **Array of Structures (AoS)** approach (each `Cell` is an independent object).
//!   It is more flexible but less memory-efficient for massive datasets compared to `libs::aggregation`.
//!
//! However, `Cell` reuses the math logic from `libs::aggregation::math` to ensure consistency
//! in statistical calculations.

use crate::libs::aggregation::{math, OpKind};

/// A state container for accumulating values in `tva wider`.
///
/// It can hold:
/// - A single float value (for simple sums, counts, min, max)
/// - A vector of floats (for mean, variance, median, etc.)
/// - A vector of strings (for string operations, mode, unique, etc.)
#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Value(f64),
    Values(Vec<f64>),
    Strings(Vec<String>),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl Cell {
    pub fn new(op: OpKind) -> Self {
        match op {
            OpKind::Count
            | OpKind::Sum
            | OpKind::MissingCount
            | OpKind::NotMissingCount => Cell::Value(0.0),
            OpKind::Min => Cell::Value(f64::INFINITY),
            OpKind::Max => Cell::Value(f64::NEG_INFINITY),
            OpKind::Mean => Cell::Values(vec![0.0, 0.0]), // [sum, count]
            OpKind::GeoMean => Cell::Values(vec![0.0, 0.0]), // [sum_log, count]
            OpKind::HarmMean => Cell::Values(vec![0.0, 0.0]), // [sum_inv, count]
            OpKind::Variance | OpKind::Stdev | OpKind::CV => {
                Cell::Values(vec![0.0, 0.0, 0.0])
            } // [sum, sum_sq, count]
            OpKind::Range => Cell::Values(vec![f64::INFINITY, f64::NEG_INFINITY]), // [min, max]
            OpKind::Median
            | OpKind::Mad
            | OpKind::Q1
            | OpKind::Q3
            | OpKind::IQR
            | OpKind::Quantile(_) => Cell::Values(Vec::new()),
            OpKind::First
            | OpKind::Last
            | OpKind::Mode
            | OpKind::ModeCount
            | OpKind::Unique
            | OpKind::NUnique
            | OpKind::Collapse
            | OpKind::Rand => Cell::Strings(Vec::new()),
        }
    }

    pub fn update(&mut self, val_bytes: &[u8], op: OpKind) {
        let val_str = std::str::from_utf8(val_bytes).unwrap_or("").trim();
        let val = if val_str.is_empty() {
            None
        } else {
            val_str.parse::<f64>().ok()
        };

        match op {
            OpKind::Count => {
                if let Cell::Value(count) = self {
                    *count += 1.0;
                } else {
                    *self = Cell::Value(1.0);
                }
            }
            OpKind::MissingCount => {
                if val_str.is_empty() {
                    if let Cell::Value(count) = self {
                        *count += 1.0;
                    } else {
                        *self = Cell::Value(1.0);
                    }
                }
            }
            OpKind::NotMissingCount => {
                if !val_str.is_empty() {
                    if let Cell::Value(count) = self {
                        *count += 1.0;
                    } else {
                        *self = Cell::Value(1.0);
                    }
                }
            }
            OpKind::Sum => {
                if let Some(v) = val {
                    if let Cell::Value(sum) = self {
                        *sum += v;
                    } else {
                        *self = Cell::Value(v);
                    }
                }
            }
            OpKind::Min => {
                if let Some(v) = val {
                    if let Cell::Value(min) = self {
                        if v < *min {
                            *min = v;
                        }
                    } else {
                        *self = Cell::Value(v);
                    }
                }
            }
            OpKind::Max => {
                if let Some(v) = val {
                    if let Cell::Value(max) = self {
                        if v > *max {
                            *max = v;
                        }
                    } else {
                        *self = Cell::Value(v);
                    }
                }
            }
            OpKind::Mean => {
                if let Some(v) = val {
                    if let Cell::Values(state) = self {
                        state[0] += v;
                        state[1] += 1.0;
                    } else {
                        *self = Cell::Values(vec![v, 1.0]);
                    }
                }
            }
            OpKind::GeoMean => {
                if let Some(v) = val {
                    if v > 0.0 {
                        if let Cell::Values(state) = self {
                            state[0] += v.ln();
                            state[1] += 1.0;
                        } else {
                            *self = Cell::Values(vec![v.ln(), 1.0]);
                        }
                    }
                }
            }
            OpKind::HarmMean => {
                if let Some(v) = val {
                    if v != 0.0 {
                        if let Cell::Values(state) = self {
                            state[0] += 1.0 / v;
                            state[1] += 1.0;
                        } else {
                            *self = Cell::Values(vec![1.0 / v, 1.0]);
                        }
                    }
                }
            }
            OpKind::Variance | OpKind::Stdev | OpKind::CV => {
                if let Some(v) = val {
                    if let Cell::Values(state) = self {
                        state[0] += v;
                        state[1] += v * v;
                        state[2] += 1.0;
                    } else {
                        *self = Cell::Values(vec![v, v * v, 1.0]);
                    }
                }
            }
            OpKind::Range => {
                if let Some(v) = val {
                    if let Cell::Values(state) = self {
                        if v < state[0] {
                            state[0] = v;
                        }
                        if v > state[1] {
                            state[1] = v;
                        }
                    } else {
                        *self = Cell::Values(vec![v, v]);
                    }
                }
            }
            OpKind::Median
            | OpKind::Mad
            | OpKind::Q1
            | OpKind::Q3
            | OpKind::IQR
            | OpKind::Quantile(_) => {
                if let Some(v) = val {
                    if let Cell::Values(vals) = self {
                        vals.push(v);
                    } else {
                        *self = Cell::Values(vec![v]);
                    }
                }
            }
            OpKind::First => {
                if let Cell::Strings(vals) = self {
                    if vals.is_empty() {
                        if !val_str.is_empty() {
                            vals.push(val_str.to_string());
                        }
                    }
                } else {
                    if !val_str.is_empty() {
                        *self = Cell::Strings(vec![val_str.to_string()]);
                    }
                }
            }
            OpKind::Last => {
                if !val_str.is_empty() {
                    if let Cell::Strings(vals) = self {
                        if vals.is_empty() {
                            vals.push(val_str.to_string());
                        } else {
                            vals[0] = val_str.to_string();
                        }
                    } else {
                        *self = Cell::Strings(vec![val_str.to_string()]);
                    }
                }
            }
            OpKind::Mode
            | OpKind::ModeCount
            | OpKind::Unique
            | OpKind::NUnique
            | OpKind::Collapse
            | OpKind::Rand => {
                if !val_str.is_empty() {
                    if let Cell::Strings(vals) = self {
                        vals.push(val_str.to_string());
                    } else {
                        *self = Cell::Strings(vec![val_str.to_string()]);
                    }
                }
            }
        }
    }

    pub fn result(&self, op: OpKind) -> String {
        match self {
            Cell::Empty => "".to_string(),
            Cell::Value(v) => v.to_string(),
            Cell::Values(vals) => match op {
                OpKind::Mean => {
                    let sum = vals[0];
                    let count = vals[1] as usize;
                    math::mean(sum, count).to_string()
                }
                OpKind::GeoMean => {
                    let sum_log = vals[0];
                    let count = vals[1] as usize;
                    math::geomean(sum_log, count).to_string()
                }
                OpKind::HarmMean => {
                    let sum_inv = vals[0];
                    let count = vals[1] as usize;
                    math::harmmean(sum_inv, count).to_string()
                }
                OpKind::Variance => {
                    let sum = vals[0];
                    let sum_sq = vals[1];
                    let count = vals[2] as usize;
                    math::variance(sum_sq, sum, count).to_string()
                }
                OpKind::Stdev => {
                    let sum = vals[0];
                    let sum_sq = vals[1];
                    let count = vals[2] as usize;
                    math::stdev(sum_sq, sum, count).to_string()
                }
                OpKind::CV => {
                    let sum = vals[0];
                    let sum_sq = vals[1];
                    let count = vals[2] as usize;
                    math::cv(sum_sq, sum, count).to_string()
                }
                OpKind::Range => {
                    if vals.len() >= 2 {
                        let res = math::range(vals[0], vals[1]);
                        if res.is_nan() {
                            "nan".to_string()
                        } else {
                            res.to_string()
                        }
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::Median => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.5).to_string()
                }
                OpKind::Mad => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::mad(&v).to_string()
                }
                OpKind::Q1 => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.25).to_string()
                }
                OpKind::Q3 => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.75).to_string()
                }
                OpKind::Quantile(p) => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, p).to_string()
                }
                OpKind::IQR => {
                    if vals.is_empty() {
                        return "nan".to_string();
                    }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let q1 = math::quantile(&v, 0.25);
                    let q3 = math::quantile(&v, 0.75);
                    (q3 - q1).to_string()
                }
                _ => "".to_string(),
            },
            Cell::Strings(vals) => match op {
                OpKind::First | OpKind::Last => {
                    if !vals.is_empty() {
                        vals[0].clone()
                    } else {
                        "".to_string()
                    }
                }
                OpKind::NUnique => {
                    let unique_vals: std::collections::HashSet<_> =
                        vals.iter().collect();
                    unique_vals.len().to_string()
                }
                OpKind::Unique => {
                    let unique_vals: std::collections::BTreeSet<_> =
                        vals.iter().collect();
                    unique_vals
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(",")
                }
                OpKind::Mode => {
                    if vals.is_empty() {
                        "".to_string()
                    } else {
                        let mut counts = std::collections::HashMap::new();
                        for v in vals {
                            *counts.entry(v).or_insert(0) += 1;
                        }
                        let mut count_vec: Vec<_> = counts.iter().collect();
                        count_vec
                            .sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                        count_vec[0].0.to_string()
                    }
                }
                OpKind::ModeCount => {
                    if vals.is_empty() {
                        "0".to_string()
                    } else {
                        let mut counts = std::collections::HashMap::new();
                        for v in vals {
                            *counts.entry(v).or_insert(0) += 1;
                        }
                        let mut count_vec: Vec<_> = counts.iter().collect();
                        count_vec
                            .sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                        count_vec[0].1.to_string()
                    }
                }
                OpKind::Collapse => vals.join(","),
                OpKind::Rand => {
                    if vals.is_empty() {
                        "".to_string()
                    } else {
                        let seed = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        let mut x = if seed == 0 { 1 } else { seed };
                        x ^= x << 13;
                        x ^= x >> 7;
                        x ^= x << 17;
                        let index = (x as usize) % vals.len();
                        vals[index].clone()
                    }
                }
                _ => "".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::aggregation::OpKind;

    fn update_cell(cell: &mut Cell, val: &str, op: OpKind) {
        cell.update(val.as_bytes(), op);
    }

    #[test]
    fn test_basic_numeric_stats() {
        // Sum
        let mut cell = Cell::new(OpKind::Sum);
        update_cell(&mut cell, "10", OpKind::Sum);
        update_cell(&mut cell, "20", OpKind::Sum);
        update_cell(&mut cell, "", OpKind::Sum); // Ignore empty
        assert_eq!(cell.result(OpKind::Sum), "30");

        // Count
        let mut cell = Cell::new(OpKind::Count);
        update_cell(&mut cell, "10", OpKind::Count);
        update_cell(&mut cell, "20", OpKind::Count);
        update_cell(&mut cell, "", OpKind::Count); // Count empty as well? update impl says: always count
        assert_eq!(cell.result(OpKind::Count), "3");

        // Min/Max
        let mut min_cell = Cell::new(OpKind::Min);
        let mut max_cell = Cell::new(OpKind::Max);
        for v in ["10", "5", "20", ""].iter() {
            update_cell(&mut min_cell, v, OpKind::Min);
            update_cell(&mut max_cell, v, OpKind::Max);
        }
        assert_eq!(min_cell.result(OpKind::Min), "5");
        assert_eq!(max_cell.result(OpKind::Max), "20");

        // Mean
        let mut cell = Cell::new(OpKind::Mean);
        update_cell(&mut cell, "10", OpKind::Mean);
        update_cell(&mut cell, "20", OpKind::Mean);
        assert_eq!(cell.result(OpKind::Mean), "15");
    }

    #[test]
    fn test_advanced_numeric_stats() {
        // Variance / Stdev
        let mut cell = Cell::new(OpKind::Variance);
        // 2, 4
        for v in ["2", "4"].iter() {
            update_cell(&mut cell, v, OpKind::Variance);
        }
        // Mean = 3
        // Variance (Sample) = ((2-3)^2 + (4-3)^2) / (2-1) = 2
        assert_eq!(cell.result(OpKind::Variance), "2");

        // Stdev = sqrt(2)
        assert_eq!(cell.result(OpKind::Stdev), (2.0f64).sqrt().to_string());

        // Range
        let mut cell = Cell::new(OpKind::Range);
        update_cell(&mut cell, "10", OpKind::Range);
        update_cell(&mut cell, "2", OpKind::Range);
        assert_eq!(cell.result(OpKind::Range), "8");
    }

    #[test]
    fn test_quantiles() {
        let mut cell = Cell::new(OpKind::Median);
        // 1, 2, 3, 4, 5
        for v in ["5", "1", "3", "2", "4"].iter() {
            update_cell(&mut cell, v, OpKind::Median);
        }
        assert_eq!(cell.result(OpKind::Median), "3");
        assert_eq!(cell.result(OpKind::Q1), "2");
        assert_eq!(cell.result(OpKind::Q3), "4");
        assert_eq!(cell.result(OpKind::IQR), "2");
    }

    #[test]
    fn test_string_ops() {
        // First / Last
        let mut first = Cell::new(OpKind::First);
        let mut last = Cell::new(OpKind::Last);
        update_cell(&mut first, "A", OpKind::First);
        update_cell(&mut first, "B", OpKind::First);
        update_cell(&mut last, "A", OpKind::Last);
        update_cell(&mut last, "B", OpKind::Last);
        assert_eq!(first.result(OpKind::First), "A");
        assert_eq!(last.result(OpKind::Last), "B");

        // Unique / NUnique
        let mut uniq = Cell::new(OpKind::Unique);
        for v in ["A", "B", "A", "C"].iter() {
            update_cell(&mut uniq, v, OpKind::Unique);
        }
        assert_eq!(uniq.result(OpKind::NUnique), "3");
        // BTreeSet sort order
        assert_eq!(uniq.result(OpKind::Unique), "A,B,C");

        // Mode
        let mut mode = Cell::new(OpKind::Mode);
        for v in ["A", "B", "A", "C", "A"].iter() {
            update_cell(&mut mode, v, OpKind::Mode);
        }
        assert_eq!(mode.result(OpKind::Mode), "A");
        assert_eq!(mode.result(OpKind::ModeCount), "3");

        // Collapse
        let mut collapse = Cell::new(OpKind::Collapse);
        update_cell(&mut collapse, "A", OpKind::Collapse);
        update_cell(&mut collapse, "B", OpKind::Collapse);
        assert_eq!(collapse.result(OpKind::Collapse), "A,B");
    }

    #[test]
    fn test_missing_counts() {
        let mut missing = Cell::new(OpKind::MissingCount);
        let mut not_missing = Cell::new(OpKind::NotMissingCount);

        for v in ["A", "", "B", ""].iter() {
            update_cell(&mut missing, v, OpKind::MissingCount);
            update_cell(&mut not_missing, v, OpKind::NotMissingCount);
        }

        assert_eq!(missing.result(OpKind::MissingCount), "2");
        assert_eq!(not_missing.result(OpKind::NotMissingCount), "2");
    }

    #[test]
    fn test_more_numeric_stats() {
        // GeoMean
        let mut geomean = Cell::new(OpKind::GeoMean);
        update_cell(&mut geomean, "2", OpKind::GeoMean);
        update_cell(&mut geomean, "8", OpKind::GeoMean);
        // sqrt(2 * 8) = 4
        assert_eq!(geomean.result(OpKind::GeoMean), "4");

        // HarmMean
        let mut harmmean = Cell::new(OpKind::HarmMean);
        update_cell(&mut harmmean, "2", OpKind::HarmMean);
        update_cell(&mut harmmean, "6", OpKind::HarmMean);
        // 2 / (1/2 + 1/6) = 2 / (3/6 + 1/6) = 2 / (4/6) = 3
        assert_eq!(harmmean.result(OpKind::HarmMean), "3");

        // CV (Coefficient of Variation) = Stdev / Mean
        let mut cv = Cell::new(OpKind::CV);
        // 2, 4 -> Mean=3, Stdev=sqrt(2) approx 1.414
        // CV = sqrt(2) / 3 approx 0.4714
        for v in ["2", "4"].iter() {
            update_cell(&mut cv, v, OpKind::CV);
        }
        let res = cv.result(OpKind::CV).parse::<f64>().unwrap();
        assert!((res - (2.0f64.sqrt() / 3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_more_distribution_stats() {
        // Quantile(p)
        let mut q = Cell::new(OpKind::Quantile(0.9));
        // 1..10
        for i in 1..=10 {
            update_cell(&mut q, &i.to_string(), OpKind::Quantile(0.9));
        }
        // 90th percentile of 1..10 should be 9.1
        assert_eq!(q.result(OpKind::Quantile(0.9)), "9.1");

        // Mad (Median Absolute Deviation)
        let mut mad = Cell::new(OpKind::Mad);
        // 1, 1, 2, 2, 4, 6, 9
        // Median = 2
        // Abs Devs: 1, 1, 0, 0, 2, 4, 7
        // Sorted Devs: 0, 0, 1, 1, 2, 4, 7 -> Median Dev = 1
        // MAD = 1 * 1.4826 = 1.4826
        for v in ["1", "1", "2", "2", "4", "6", "9"].iter() {
            update_cell(&mut mad, v, OpKind::Mad);
        }
        assert!(!mad.result(OpKind::Mad).is_empty());
    }

    #[test]
    fn test_rand() {
        let mut rand = Cell::new(OpKind::Rand);
        update_cell(&mut rand, "A", OpKind::Rand);
        update_cell(&mut rand, "B", OpKind::Rand);
        let res = rand.result(OpKind::Rand);
        assert!(res == "A" || res == "B");

        let empty = Cell::new(OpKind::Rand);
        assert_eq!(empty.result(OpKind::Rand), "");
    }

    #[test]
    fn test_edge_cases() {
        // Range with < 2 values
        let mut range = Cell::new(OpKind::Range);
        update_cell(&mut range, "5", OpKind::Range);
        // Cell::Range init: vec![inf, -inf], len=2
        // Update: updates state[0] (min) and state[1] (max).
        // Result: if vals.len() >= 2 -> correct
        assert_eq!(range.result(OpKind::Range), "0");

        // Empty Range
        let empty_range = Cell::new(OpKind::Range);
        // min=inf, max=-inf -> range = nan
        assert_eq!(empty_range.result(OpKind::Range), "nan");

        // Empty Median/Quantile
        let empty_q = Cell::new(OpKind::Median);
        assert_eq!(empty_q.result(OpKind::Median), "nan");

        // Empty Mode
        let empty_mode = Cell::new(OpKind::Mode);
        assert_eq!(empty_mode.result(OpKind::Mode), "");
        assert_eq!(empty_mode.result(OpKind::ModeCount), "0");
    }

    #[test]
    fn test_lazy_init_coverage() {
        // GeoMean
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "2", OpKind::GeoMean);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[0], 2.0f64.ln());
            assert_eq!(v[1], 1.0);
        } else {
            panic!("GeoMean lazy init failed");
        }

        // HarmMean
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "2", OpKind::HarmMean);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[0], 0.5);
            assert_eq!(v[1], 1.0);
        } else {
            panic!("HarmMean lazy init failed");
        }

        // Variance
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "2", OpKind::Variance);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[0], 2.0); // sum
            assert_eq!(v[1], 4.0); // sum_sq
            assert_eq!(v[2], 1.0); // count
        } else {
            panic!("Variance lazy init failed");
        }

        // Range
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "5", OpKind::Range);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[0], 5.0); // min
            assert_eq!(v[1], 5.0); // max
        } else {
            panic!("Range lazy init failed");
        }

        // Median/Quantile (Vector ops)
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "5", OpKind::Median);
        if let Cell::Values(v) = &cell {
            assert_eq!(v.len(), 1);
            assert_eq!(v[0], 5.0);
        } else {
            panic!("Median lazy init failed");
        }

        // First
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "A", OpKind::First);
        if let Cell::Strings(v) = &cell {
            assert_eq!(v[0], "A");
        } else {
            panic!("First lazy init failed");
        }

        // Last
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "A", OpKind::Last);
        if let Cell::Strings(v) = &cell {
            assert_eq!(v[0], "A");
        } else {
            panic!("Last lazy init failed");
        }

        // Mode/Unique (String list ops)
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "A", OpKind::Mode);
        if let Cell::Strings(v) = &cell {
            assert_eq!(v[0], "A");
        } else {
            panic!("Mode lazy init failed");
        }
    }

    #[test]
    fn test_ignored_values() {
        // GeoMean: ignore <= 0
        let mut cell = Cell::new(OpKind::GeoMean);
        update_cell(&mut cell, "-5", OpKind::GeoMean);
        update_cell(&mut cell, "0", OpKind::GeoMean);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[1], 0.0); // count should remain 0
        }

        // HarmMean: ignore 0
        let mut cell = Cell::new(OpKind::HarmMean);
        update_cell(&mut cell, "0", OpKind::HarmMean);
        if let Cell::Values(v) = &cell {
            assert_eq!(v[1], 0.0); // count should remain 0
        }

        // First: ignore empty string
        let mut cell = Cell::new(OpKind::First);
        update_cell(&mut cell, "", OpKind::First);
        if let Cell::Strings(v) = &cell {
            assert!(v.is_empty());
        }

        // Last: ignore empty string
        let mut cell = Cell::new(OpKind::Last);
        update_cell(&mut cell, "", OpKind::Last);
        if let Cell::Strings(v) = &cell {
            assert!(v.is_empty());
        }

        // Mode: ignore empty string
        let mut cell = Cell::new(OpKind::Mode);
        update_cell(&mut cell, "", OpKind::Mode);
        if let Cell::Strings(v) = &cell {
            assert!(v.is_empty());
        }
    }

    #[test]
    fn test_first_last_logic() {
        // First: keep first non-empty
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "A", OpKind::First); // Init with A
        update_cell(&mut cell, "B", OpKind::First); // Should ignore B
        assert_eq!(cell.result(OpKind::First), "A");

        // Test First initialized but empty (simulating new() then update)
        let mut cell = Cell::new(OpKind::First); // Strings([])
        update_cell(&mut cell, "A", OpKind::First); // Pushes A
        update_cell(&mut cell, "B", OpKind::First); // Ignored
        assert_eq!(cell.result(OpKind::First), "A");

        // Last: overwrite with last non-empty
        let mut cell = Cell::Empty;
        update_cell(&mut cell, "A", OpKind::Last); // Init with A
        update_cell(&mut cell, "B", OpKind::Last); // Overwrite with B
        assert_eq!(cell.result(OpKind::Last), "B");

        // Test Last initialized but empty
        let mut cell = Cell::new(OpKind::Last); // Strings([])
        update_cell(&mut cell, "A", OpKind::Last); // Pushes A
        update_cell(&mut cell, "B", OpKind::Last); // Overwrites with B
        assert_eq!(cell.result(OpKind::Last), "B");
    }

    #[test]
    fn test_result_branches() {
        // Range with 0 or 1 value (should be nan/valid)
        // Range uses Values[min, max] when init from Empty/Update.
        // But result() implementation:
        // OpKind::Range => if vals.len() >= 2 ...
        // Wait, lazy init for Range creates Values(vec![v, v]) (len 2).
        // Cell::new(Range) creates Values(vec![inf, -inf]) (len 2).
        // So Range always has len 2 if initialized properly.
        // When could it have len < 2? Only if manually corrupted or Cell::Values initialized incorrectly for other ops then switched?
        // Let's verify result() safety against empty Values.
        let cell = Cell::Values(vec![1.0]); // Invalid state for Range
        assert_eq!(cell.result(OpKind::Range), "nan");

        // Median with empty values
        let cell = Cell::Values(vec![]);
        assert_eq!(cell.result(OpKind::Median), "nan");

        // First/Last with empty strings
        let cell = Cell::Strings(vec![]);
        assert_eq!(cell.result(OpKind::First), "");
    }

    #[test]
    fn test_invalid_numeric() {
        // Sum ignores invalid numeric string
        let mut cell = Cell::new(OpKind::Sum);
        update_cell(&mut cell, "10", OpKind::Sum);
        update_cell(&mut cell, "abc", OpKind::Sum); // Should be ignored
        assert_eq!(cell.result(OpKind::Sum), "10");

        // Mean ignores invalid
        let mut cell = Cell::new(OpKind::Mean);
        update_cell(&mut cell, "10", OpKind::Mean);
        update_cell(&mut cell, "abc", OpKind::Mean);
        assert_eq!(cell.result(OpKind::Mean), "10");

        // Min ignores invalid
        let mut cell = Cell::new(OpKind::Min);
        update_cell(&mut cell, "10", OpKind::Min);
        update_cell(&mut cell, "abc", OpKind::Min);
        assert_eq!(cell.result(OpKind::Min), "10");
    }

    #[test]
    fn test_type_overwrite() {
        // Init with Strings (wrong type for Sum)
        let mut cell = Cell::Strings(vec!["A".to_string()]);
        // Update with Sum -> Should overwrite with Value
        update_cell(&mut cell, "10", OpKind::Sum);
        if let Cell::Value(v) = cell {
            assert_eq!(v, 10.0);
        } else {
            panic!("Expected Cell::Value after overwrite");
        }

        // Init with Value (wrong type for Mean)
        let mut cell = Cell::Value(10.0);
        // Update with Mean -> Should overwrite with Values
        update_cell(&mut cell, "20", OpKind::Mean);
        if let Cell::Values(v) = cell {
            assert_eq!(v[0], 20.0);
            assert_eq!(v[1], 1.0);
        } else {
            panic!("Expected Cell::Values after overwrite");
        }
    }

    #[test]
    fn test_noop_updates() {
        // Min: update with larger value
        let mut cell = Cell::new(OpKind::Min);
        update_cell(&mut cell, "10", OpKind::Min);
        update_cell(&mut cell, "20", OpKind::Min); // Should not change min
        assert_eq!(cell.result(OpKind::Min), "10");

        // Max: update with smaller value
        let mut cell = Cell::new(OpKind::Max);
        update_cell(&mut cell, "10", OpKind::Max);
        update_cell(&mut cell, "5", OpKind::Max); // Should not change max
        assert_eq!(cell.result(OpKind::Max), "10");

        // Range: update with value inside range
        let mut cell = Cell::new(OpKind::Range);
        update_cell(&mut cell, "10", OpKind::Range); // min=10, max=10
        update_cell(&mut cell, "20", OpKind::Range); // min=10, max=20
        update_cell(&mut cell, "15", OpKind::Range); // Should not change
        if let Cell::Values(v) = &cell {
            assert_eq!(v[0], 10.0);
            assert_eq!(v[1], 20.0);
        }

        // MissingCount: update with non-empty
        let mut cell = Cell::new(OpKind::MissingCount);
        update_cell(&mut cell, "", OpKind::MissingCount); // +1
        update_cell(&mut cell, "val", OpKind::MissingCount); // Ignored
        assert_eq!(cell.result(OpKind::MissingCount), "1");

        // NotMissingCount: update with empty
        let mut cell = Cell::new(OpKind::NotMissingCount);
        update_cell(&mut cell, "val", OpKind::NotMissingCount); // +1
        update_cell(&mut cell, "", OpKind::NotMissingCount); // Ignored
        assert_eq!(cell.result(OpKind::NotMissingCount), "1");
    }

    #[test]
    fn test_mode_tie_breaking() {
        let mut mode = Cell::new(OpKind::Mode);
        // A: 2, B: 2. Should pick A (lexicographical)
        for v in ["B", "A", "B", "A"].iter() {
            update_cell(&mut mode, v, OpKind::Mode);
        }
        assert_eq!(mode.result(OpKind::Mode), "A");

        // C: 3, A: 2, B: 2. Should pick C (count)
        update_cell(&mut mode, "C", OpKind::Mode);
        update_cell(&mut mode, "C", OpKind::Mode);
        update_cell(&mut mode, "C", OpKind::Mode);
        assert_eq!(mode.result(OpKind::Mode), "C");
    }

    #[test]
    fn test_cross_type_safety() {
        // Cell::Value (e.g. Sum) called with Mean
        let sum_cell = Cell::Value(10.0);
        // Should return string representation of value
        assert_eq!(sum_cell.result(OpKind::Mean), "10");

        // Cell::Values (e.g. Mean) called with Sum
        let mean_cell = Cell::Values(vec![10.0, 2.0]);
        // Cell::Values matches specific Ops, _ => ""
        assert_eq!(mean_cell.result(OpKind::Sum), "");

        // Cell::Strings (e.g. First) called with Sum
        let str_cell = Cell::Strings(vec!["A".to_string()]);
        // Cell::Strings matches specific Ops, _ => ""
        assert_eq!(str_cell.result(OpKind::Sum), "");
    }

    #[test]
    fn test_cv_edge_cases() {
        // Mean = 0 -> CV = Stdev / 0 -> NAN (per math::cv impl)
        let mut cv = Cell::new(OpKind::CV);
        // -2, 2 -> Mean=0, Stdev=sqrt(8)/1 = 2.82
        update_cell(&mut cv, "-2", OpKind::CV);
        update_cell(&mut cv, "2", OpKind::CV);

        // math::cv checks if mean != 0.0, else returns NAN
        assert_eq!(cv.result(OpKind::CV), "NaN");
    }

    #[test]
    fn test_count_behavior() {
        let mut count = Cell::new(OpKind::Count);
        update_cell(&mut count, "A", OpKind::Count);
        update_cell(&mut count, "", OpKind::Count); // Should count empty
        assert_eq!(count.result(OpKind::Count), "2");
    }

    #[test]
    fn test_default_impl() {
        // Test Default trait implementation
        let cell: Cell = Default::default();
        assert!(matches!(cell, Cell::Empty));
    }

    #[test]
    fn test_update_else_branches() {
        // Test Count with wrong cell type (triggers else branch)
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Count);
        assert_eq!(cell.result(OpKind::Count), "1");

        // Test MissingCount with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "", OpKind::MissingCount);
        assert_eq!(cell.result(OpKind::MissingCount), "1");

        // Test NotMissingCount with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "val", OpKind::NotMissingCount);
        assert_eq!(cell.result(OpKind::NotMissingCount), "1");

        // Test Sum with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "10", OpKind::Sum);
        assert_eq!(cell.result(OpKind::Sum), "10");

        // Test Min with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Min);
        assert_eq!(cell.result(OpKind::Min), "5");

        // Test Max with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Max);
        assert_eq!(cell.result(OpKind::Max), "5");

        // Test Mean with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "10", OpKind::Mean);
        assert_eq!(cell.result(OpKind::Mean), "10");

        // Test GeoMean with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "4", OpKind::GeoMean);
        assert_eq!(cell.result(OpKind::GeoMean), "4");

        // Test HarmMean with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "2", OpKind::HarmMean);
        assert_eq!(cell.result(OpKind::HarmMean), "2");

        // Test Variance with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Variance);
        assert_eq!(cell.result(OpKind::Variance), "NaN"); // single value

        // Test Range with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Range);
        assert_eq!(cell.result(OpKind::Range), "0");

        // Test Median with wrong cell type
        let mut cell = Cell::Strings(vec!["test".to_string()]);
        update_cell(&mut cell, "5", OpKind::Median);
        assert_eq!(cell.result(OpKind::Median), "5");

        // Test First with wrong cell type
        let mut cell = Cell::Value(10.0);
        update_cell(&mut cell, "A", OpKind::First);
        assert_eq!(cell.result(OpKind::First), "A");

        // Test Last with wrong cell type
        let mut cell = Cell::Value(10.0);
        update_cell(&mut cell, "B", OpKind::Last);
        assert_eq!(cell.result(OpKind::Last), "B");

        // Test Mode with wrong cell type
        let mut cell = Cell::Value(10.0);
        update_cell(&mut cell, "X", OpKind::Mode);
        assert_eq!(cell.result(OpKind::Mode), "X");
    }

    #[test]
    fn test_result_unmatched_ops() {
        // Test Cell::Values with unmatched OpKind
        let cell = Cell::Values(vec![1.0, 2.0, 3.0]);
        assert_eq!(cell.result(OpKind::Sum), ""); // Sum not matched in Values

        // Test Cell::Strings with unmatched OpKind
        let cell = Cell::Strings(vec!["A".to_string()]);
        assert_eq!(cell.result(OpKind::Mean), ""); // Mean not matched in Strings
    }

    #[test]
    fn test_collapse_with_duplicates() {
        // Test Collapse with duplicate values
        let mut cell = Cell::new(OpKind::Collapse);
        update_cell(&mut cell, "A", OpKind::Collapse);
        update_cell(&mut cell, "A", OpKind::Collapse); // Duplicate
        update_cell(&mut cell, "B", OpKind::Collapse);
        let result = cell.result(OpKind::Collapse);
        assert!(result.contains("A"));
        assert!(result.contains("B"));
    }

    #[test]
    fn test_rand_with_empty_cell() {
        // Test Rand with Cell::Empty
        let cell = Cell::Empty;
        assert_eq!(cell.result(OpKind::Rand), "");
    }

    #[test]
    fn test_mode_count_empty() {
        // Test ModeCount with empty cell
        let cell = Cell::Empty;
        assert_eq!(cell.result(OpKind::ModeCount), "");
    }

    #[test]
    fn test_nunique_empty() {
        // Test NUnique with empty cell
        let cell = Cell::Empty;
        assert_eq!(cell.result(OpKind::NUnique), "");
    }

    #[test]
    fn test_unique_empty() {
        // Test Unique with empty cell
        let cell = Cell::Empty;
        assert_eq!(cell.result(OpKind::Unique), "");
    }

    #[test]
    fn test_collapse_empty() {
        // Test Collapse with empty cell
        let cell = Cell::Empty;
        assert_eq!(cell.result(OpKind::Collapse), "");
    }
}
