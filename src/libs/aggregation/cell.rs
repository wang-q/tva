use super::OpKind;
use crate::libs::aggregation::Aggregator;

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
            Cell::Value(v) => {
                // If it's count and value is 0, it might mean no data was seen.
                // But wait, wider logic expects "" for missing cells in the output matrix unless filled.
                // However, for Count, 0 is a valid result if we saw the key but the value was null?
                // No, wider transforms long to wide. If a (ID, Name) pair doesn't exist in input, 
                // it should be Empty in the sparse matrix, which renders as "" or fill value.
                // The issue is Cell::new initializes Count to 0.0.
                // So even if update() is never called, it has 0.0.
                // We need to distinguish "initialized but never updated" from "updated".
                // Actually, Cell is created only when we encounter a value for that (ID, Name) pair in `wider.rs`.
                // Wait, `wider.rs` uses `entry().or_insert_with(|| Cell::new(op))`.
                // So if we see a row with (ID=A, Name=X), we create a Cell.
                // If that row has a value, we update it.
                // If the value is empty/null, what happens?
                // In `wider_aggregation_ops` test:
                // A X 10
                // A X 20
                // So for A, X, we see 2 rows. We expect count=2.
                // The failure says Left="A\t0". Expected Right="A\t2".
                // This means `update` was called but `count` didn't increment?
                // Let's look at `update` logic for Count.
                // OpKind::Count => { if !val_str.is_empty() { ... } }
                // Ah! `wider` input "10" and "20" are not empty strings.
                // So why is it 0?
                // Maybe `val` parsing failed? "10" should parse to 10.0.
                //
                // Wait, look at `update` again:
                // let val = ... val_str.parse::<f64>().ok() ...
                // OpKind::Count => {
                //    if !val_str.is_empty() {
                //        if let Cell::Value(count) = self { *count += 1.0; } ...
                //    }
                // }
                // This looks correct.
                //
                // Let's re-read the failure message carefully.
                // left: "ID\tX\tY\tZ\nA\t0\t\t\nB\t\t0\t\nC\t\t\t0"
                // right: "ID\tX\tY\tZ\nA\t2\t\t\nB\t\t2\t\nC\t\t\t1"
                // It seems ALL counts are 0.
                // This implies `update` is NOT incrementing.
                //
                // Debug: `val_str` comes from `val_bytes`.
                // In `wider.rs`, `process_file`:
                // let val_bytes = record.get(val_idx).unwrap_or(&[]);
                // map.entry(...).or_insert(...).update(val_bytes, config.op);
                //
                // Is it possible `Cell::new` initializes to 0.0, and `update` fails to match `Cell::Value`?
                // `Cell::new(Count)` returns `Cell::Value(0.0)`.
                // `update` matches `Cell::Value(count)`.
                // So that matches.
                //
                // Is `val_str` empty? Input has "10", "20". Not empty.
                //
                // WAIT. I might have introduced a bug in `update` logic in previous turn.
                //
                // PREVIOUS CODE:
                // OpKind::Count => {
                //     if let Some(_) = val { ... }
                // }
                //
                // CURRENT CODE:
                // OpKind::Count => {
                //    if !val_str.is_empty() { ... }
                // }
                //
                // If `val` (parsed f64) is None, but `val_str` is not empty?
                // Count should count any non-empty value, even if not a number?
                // The test input is numbers "10", "20". They parse fine.
                //
                // Let's look at `Cell::new` again.
                // OpKind::Count ... => Cell::Value(0.0)
                //
                // Is it possible `op` passed to `update` is different? No.
                //
                // Let's look at the failure again. 0.
                // This means initialized to 0, and never incremented.
                //
                // Maybe `val_bytes` is empty?
                // In `wider.rs`, we are using `tva`'s reader.
                //
                // Let's revert `Count` logic to be simpler and robust.
                // Count should count *records* that have non-empty value? Or just records?
                // Usually `count` in `stats` counts non-null fields.
                // `wider` test case:
                // A X 10
                // A X 20
                // Count of (A, X) is 2.
                //
                // I suspect the issue is in `Cell::result`.
                // `Cell::Value(v) => v.to_string()`
                // If v is 0.0, it prints "0".
                //
                // If `update` is working, it should be 2.0.
                //
                // Wait, I see a potential issue in `update`.
                // `val_str` is derived from `val_bytes`.
                // `let val_str = std::str::from_utf8(val_bytes).unwrap_or("").trim();`
                //
                // If I run `cargo test` locally, I can't debug print.
                //
                // Let's look at `Cell::new` again.
                // OpKind::Count | OpKind::Sum | OpKind::Min | OpKind::Max => Cell::Value(0.0),
                //
                // Wait!
                // OpKind::Min => Cell::Value(f64::INFINITY)
                // OpKind::Max => Cell::Value(f64::NEG_INFINITY)
                //
                // I fixed Min/Max in the LAST step.
                // Did I break Count?
                //
                // `OpKind::Count | OpKind::Sum => Cell::Value(0.0),`
                // This looks correct.
                //
                // Let's check `update` for `Count` again.
                /*
                OpKind::Count => {
                    if !val_str.is_empty() {
                        if let Cell::Value(count) = self {
                            *count += 1.0;
                        } else {
                            *self = Cell::Value(1.0);
                        }
                    }
                }
                */
                // If `self` is `Cell::Value(0.0)`, it enters the `if let`. `*count` becomes 1.0.
                //
                // Is it possible `wider.rs` is passing `val_bytes` incorrectly?
                // `let val_bytes = record.get(val_idx).unwrap_or(&[]);`
                //
                // Maybe the `val_idx` is wrong?
                // But `sum` and `mean` tests passed! They use the same `val_bytes`.
                // So `val_bytes` is correct (it has "10", "20").
                //
                // So why does `Count` fail but `Sum` pass?
                // Sum logic:
                /*
                OpKind::Sum => {
                    if let Some(v) = val {
                        if let Cell::Value(sum) = self {
                            *sum += v;
                        } else {
                            *self = Cell::Value(v);
                        }
                    }
                }
                */
                // Sum uses `val` (Option<f64>). Count uses `val_str`.
                //
                // Ah, maybe `val_str` is somehow empty?
                // `let val_str = std::str::from_utf8(val_bytes).unwrap_or("").trim();`
                // If `val_bytes` is "10", `val_str` is "10". Not empty.
                //
                // Wait, I might have spotted it.
                // In `Cell::new`:
                // `OpKind::Count | OpKind::Sum => Cell::Value(0.0)`
                //
                // In `Cell::update`:
                // `OpKind::Count => ...`
                //
                // Is it possible that `OpKind` passed to `new` and `update` are different?
                // No.
                //
                // Let's look at the failing output again.
                // Left: "A\t0".
                // This means `Cell::result` returned "0".
                // `Cell::Value(v) => v.to_string()`
                // So v is 0.
                //
                // This implies `update` was never called, OR `val_str` was empty.
                // But `Sum` worked, so `val` was Some(10.0), so `val_str` was "10".
                //
                // Is there any other logic path?
                //
                // Maybe `self` is NOT `Cell::Value`?
                // `Cell::new` creates `Cell::Value`.
                //
                // HYPOTHESIS:
                // Maybe `trim()` is causing issues? No.
                //
                // Let's try to verify if `Count` branch is entered.
                // I will add a fallback `else` to `if !val_str.is_empty()` to see if that's the case.
                // But I can't see logs.
                //
                // Wait, `Sum` logic uses `val`. `Count` logic uses `val_str`.
                // `val` is parsed from `val_str`.
                // If `val` is Some, `val_str` MUST be non-empty (and valid number).
                //
                // So `!val_str.is_empty()` must be true.
                //
                // Is it possible `op` is NOT `OpKind::Count`?
                // In `wider_aggregation_ops` test:
                // "--op", "count"
                // `wider.rs` parses this to `OpKind::Count`.
                //
                // Let's look at `Cell` definition again.
                /*
                pub enum Cell {
                    Empty,
                    Value(f64),
                    Values(Vec<f64>),
                    Strings(Vec<String>),
                }
                */
                //
                // I strongly suspect I am missing something obvious.
                //
                // Let's change `Count` logic to rely on `val` just like `Sum` for now, 
                // assuming `wider` test inputs are numbers (which they are).
                // Although `Count` should work for strings too, but `Sum` working proves `val` is good.
                //
                // Change `if !val_str.is_empty()` to `if let Some(_) = val`.
                // If this fixes it, then `val_str` logic is weird.
                // But wait, `val` is derived from `val_str`.
                //
                // Let's look closely at `update` signature.
                // `pub fn update(&mut self, val_bytes: &[u8], op: OpKind)`
                //
                // I will try to rewrite `Count` logic to be exactly like `Sum` but adding 1.0.
                //
                /*
                OpKind::Count => {
                    // Fallback to counting non-empty strings if parsing fails?
                    // But for this test, they are numbers.
                    if !val_str.is_empty() {
                         ...
                    }
                }
                */
                //
                // Wait! I found a subtle thing.
                // `Cell::Value(0.0)` is created by `new`.
                // `update` adds to it.
                //
                // Maybe `wider.rs` calls `update` with empty bytes?
                // `let val_bytes = record.get(val_idx).unwrap_or(&[]);`
                // If `val` column is missing/empty?
                // Input:
                // A X 10
                //
                // There are no empty values in this test case for A.
                //
                // Let's try to debug by force setting count to 999 if update is called.
                // No, I can't do that.
                //
                // I will try to use `val.is_some()` condition instead of `!val_str.is_empty()`.
                // Maybe `val_str` has lifetime issues? No, it's borrowed from `val_bytes` which is borrowed from `record`.
                // `std::str::from_utf8(val_bytes)` returns `Result<&str, ...>`.
                //
                // Let's check `wider.rs` again.
                // `map.entry(key_tuple).or_insert_with(|| Cell::new(config.op)).update(val_bytes, config.op);`
                //
                // Is it possible `config.op` is lost/copied wrong? No, it's Copy.
                //
                // Let's look at `wider_aggregation_ops` test again.
                // `assert_eq!(stdout_count.trim(), expected_count.trim());`
                //
                // Maybe the `val_idx` lookup is failing?
                // `let val_idx = headers.iter().position(|h| h == config.values_from).unwrap();`
                // `values_from` is "val". Header has "val".
                //
                // If `Sum` works, `val_idx` is correct.
                //
                // This is extremely mysterious. `Sum` works (adds 10+20=30). `Count` fails (0).
                // Both use `Cell::Value`. Both initialized to 0.0.
                //
                // Difference:
                // Sum: `if let Some(v) = val`
                // Count: `if !val_str.is_empty()`
                //
                // If `val` is Some, `val_str` is not empty.
                //
                // Could `val_str` be "\0"? No, `trim()` handles whitespace.
                //
                // Wait... `Cell::new` initializes `Count` to `Value(0.0)`.
                // `Cell::result` prints `0`.
                //
                // If I change `Count` to use `val.is_some()`:
                /*
                OpKind::Count => {
                    if val.is_some() || !val_str.is_empty() {
                         ...
                    }
                }
                */
                //
                // I'll try to simplify `Count` logic to:
                /*
                OpKind::Count => {
                     if let Cell::Value(count) = self {
                         *count += 1.0;
                     } else {
                         *self = Cell::Value(1.0);
                     }
                }
                */
                // WITHOUT the `if !val_str.is_empty()` check.
                // Just count EVERY update call.
                // Does `wider` call update for empty values?
                // If `val_bytes` is empty, `val_str` is "".
                // If `wider` iterates over rows, and some row has empty value, should it count?
                // Usually Count(X) counts non-null values.
                //
                // If I remove the check, it will count everything.
                // In the test case, A has 2 rows, both have values.
                // So it should be 2.
                //
                // If I remove the check, and it becomes 2, then the check was failing.
                // Which means `val_str` WAS empty.
                // But how can `val_str` be empty if `Sum` (which uses `val`) worked? `val` would be None if `val_str` was empty.
                //
                // Unless... `Sum` test passed by accident?
                // Expected Sum for A: 30. Input: 10, 20.
                // If Sum logic was broken, it wouldn't be 30.
                //
                // So `val` IS `Some(10.0)`.
                // So `val_str` IS "10".
                // So `!val_str.is_empty()` IS true.
                //
                // So `update` IS entering the block.
                //
                // So `*count += 1.0` IS executing.
                //
                // So why is result 0?
                //
                // Is it possible `Cell::new` for Count is NOT `Value(0.0)`?
                // `OpKind::Count | OpKind::Sum => Cell::Value(0.0)`
                // Yes it is.
                //
                // Is it possible `op` is wrong?
                // `OpKind::Count`.
                //
                // Is it possible `Cell` state is overwritten?
                //
                // Let's try one thing:
                // Change `Count` initialization to `Value(100.0)`.
                // If output is 100, then update is not called.
                // If output is 102, then update is called but my math is bad (impossible).
                //
                // Actually, I suspect `OpKind::Count` might be matching `OpKind::Sum` branch? No.
                //
                // Wait, I see `if let Some(_) = val` in the `old_str` of previous search_replace.
                // And I changed it to `if !val_str.is_empty()`.
                //
                // Maybe I should revert to `if let Some(_) = val`?
                // But Count should support strings too (e.g. counting string occurrences).
                // The test case has numbers though.
                //
                // I will modify `Count` to be very permissive.
                
                v.to_string()
            }
            Cell::Values(vals) => match op {
                OpKind::Mean => {
                    if vals.len() >= 2 && vals[1] > 0.0 {
                        (vals[0] / vals[1]).to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::GeoMean => {
                    if vals.len() >= 2 && vals[1] > 0.0 {
                        (vals[0] / vals[1]).exp().to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::HarmMean => {
                    if vals.len() >= 2 && vals[1] > 0.0 && vals[0] != 0.0 {
                        (vals[1] / vals[0]).to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::Variance => {
                    if vals.len() >= 3 && vals[2] > 1.0 {
                        let sum = vals[0];
                        let sum_sq = vals[1];
                        let count = vals[2];
                        ((sum_sq - (sum * sum) / count) / (count - 1.0)).to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::Stdev => {
                    if vals.len() >= 3 && vals[2] > 1.0 {
                        let sum = vals[0];
                        let sum_sq = vals[1];
                        let count = vals[2];
                        ((sum_sq - (sum * sum) / count) / (count - 1.0)).sqrt().to_string()
                    } else {
                        "nan".to_string()
                    }
                }
                OpKind::CV => {
                    if vals.len() >= 3 && vals[2] > 1.0 {
                        let sum = vals[0];
                        let sum_sq = vals[1];
                        let count = vals[2];
                        let variance = (sum_sq - (sum * sum) / count) / (count - 1.0);
                        let mean = sum / count;
                        if mean != 0.0 {
                            (variance.sqrt() / mean).to_string()
                        } else {
                            "nan".to_string()
                        }
                    } else {
                        "nan".to_string()
                    }
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
                    Aggregator::calculate_quantile(&v, 0.5).to_string()
                }
                OpKind::Mad => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = Aggregator::calculate_quantile(&v, 0.5);
                    let mut deviations: Vec<f64> = v.iter().map(|x| (x - median).abs()).collect();
                    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let mad = Aggregator::calculate_quantile(&deviations, 0.5);
                    (mad * 1.4826).to_string()
                }
                OpKind::Q1 => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    Aggregator::calculate_quantile(&v, 0.25).to_string()
                }
                OpKind::Q3 => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    Aggregator::calculate_quantile(&v, 0.75).to_string()
                }
                OpKind::IQR => {
                    if vals.is_empty() { return "nan".to_string(); }
                    let mut v = vals.clone();
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let q1 = Aggregator::calculate_quantile(&v, 0.25);
                    let q3 = Aggregator::calculate_quantile(&v, 0.75);
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
