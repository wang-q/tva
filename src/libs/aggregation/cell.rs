use super::OpKind;
use crate::libs::aggregation::math;

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
            OpKind::Count | OpKind::Sum => Cell::Value(0.0),
            OpKind::Min => Cell::Value(f64::INFINITY),
            OpKind::Max => Cell::Value(f64::NEG_INFINITY),
            OpKind::Mean => Cell::Values(vec![0.0, 0.0]), // [sum, count]
            OpKind::GeoMean => Cell::Values(vec![0.0, 0.0]), // [sum_log, count]
            OpKind::HarmMean => Cell::Values(vec![0.0, 0.0]), // [sum_inv, count]
            OpKind::Variance | OpKind::Stdev | OpKind::CV => Cell::Values(vec![0.0, 0.0, 0.0]), // [sum, sum_sq, count]
            OpKind::Range => Cell::Values(vec![f64::INFINITY, f64::NEG_INFINITY]), // [min, max]
            OpKind::Median | OpKind::Mad | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => Cell::Values(Vec::new()),
            OpKind::First | OpKind::Last | OpKind::Mode | OpKind::Unique | OpKind::NUnique | OpKind::Collapse | OpKind::Rand => Cell::Strings(Vec::new()),
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
                        if v < state[0] { state[0] = v; }
                        if v > state[1] { state[1] = v; }
                    } else {
                        *self = Cell::Values(vec![v, v]);
                    }
                }
            }
            OpKind::Median | OpKind::Mad | OpKind::Q1 | OpKind::Q3 | OpKind::IQR => {
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
            OpKind::Mode | OpKind::Unique | OpKind::NUnique | OpKind::Collapse | OpKind::Rand => {
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
                    if vals.len() >= 2 && vals[0] != f64::INFINITY && vals[1] != f64::NEG_INFINITY {
                        (vals[1] - vals[0]).to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::Median => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.5).to_string()
                }
                OpKind::Mad => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::mad(&v).to_string()
                }
                OpKind::Q1 => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.25).to_string()
                }
                OpKind::Q3 => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    math::quantile(&v, 0.75).to_string()
                }
                OpKind::IQR => {
                    if vals.is_empty() { return "nan".to_string(); }
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
                    let unique_vals: std::collections::HashSet<_> = vals.iter().collect();
                    unique_vals.len().to_string()
                }
                OpKind::Unique => {
                    let unique_vals: std::collections::BTreeSet<_> = vals.iter().collect();
                    unique_vals.into_iter().cloned().collect::<Vec<_>>().join(",")
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
                        count_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
                        count_vec[0].0.to_string()
                    }
                }
                OpKind::Collapse => {
                    vals.join(",")
                }
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
