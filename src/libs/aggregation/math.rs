//! Common mathematical functions for aggregation.
//! These functions are pure and operate on primitive types or slices,
//! agnostic of the underlying storage layout (SoA or AoS).

/// Calculates the mean from sum and count.
#[inline]
pub fn mean(sum: f64, count: usize) -> f64 {
    if count > 0 {
        sum / count as f64
    } else {
        f64::NAN
    }
}

/// Calculates the geometric mean from sum of logs and count.
#[inline]
pub fn geomean(sum_log: f64, count: usize) -> f64 {
    if count > 0 {
        (sum_log / count as f64).exp()
    } else {
        f64::NAN
    }
}

/// Calculates the harmonic mean from sum of inverses and count.
#[inline]
pub fn harmmean(sum_inv: f64, count: usize) -> f64 {
    if count > 0 && sum_inv != 0.0 {
        count as f64 / sum_inv
    } else {
        f64::NAN
    }
}

/// Calculates the sample variance.
#[inline]
pub fn variance(sum_sq: f64, sum: f64, count: usize) -> f64 {
    if count > 1 {
        let mean = sum / count as f64;
        (sum_sq - (sum * mean)) / (count as f64 - 1.0)
    } else {
        f64::NAN
    }
}

/// Calculates the sample standard deviation.
#[inline]
pub fn stdev(sum_sq: f64, sum: f64, count: usize) -> f64 {
    variance(sum_sq, sum, count).sqrt()
}

/// Calculates the coefficient of variation.
#[inline]
pub fn cv(sum_sq: f64, sum: f64, count: usize) -> f64 {
    if count > 1 {
        let mean = sum / count as f64;
        if mean != 0.0 {
            stdev(sum_sq, sum, count) / mean
        } else {
            f64::NAN
        }
    } else {
        f64::NAN
    }
}

/// Calculates the quantile value at probability p.
/// The input slice MUST be sorted.
pub fn quantile(sorted_vals: &[f64], p: f64) -> f64 {
    let len = sorted_vals.len();
    if len == 0 {
        return f64::NAN;
    }
    if len == 1 {
        return sorted_vals[0];
    }
    let pos = p * (len - 1) as f64;
    let i = pos.floor() as usize;
    let fract = pos - i as f64;
    if i >= len - 1 {
        sorted_vals[len - 1]
    } else {
        sorted_vals[i] * (1.0 - fract) + sorted_vals[i + 1] * fract
    }
}

/// Calculates the Median Absolute Deviation (MAD).
/// The input slice MUST be sorted.
/// This function allocates a new vector for deviations.
pub fn mad(sorted_vals: &[f64]) -> f64 {
    if sorted_vals.is_empty() {
        return f64::NAN;
    }
    let median = quantile(sorted_vals, 0.5);
    
    let mut deviations: Vec<f64> = sorted_vals
        .iter()
        .map(|v| (v - median).abs())
        .collect();
    // We need to sort deviations to find their median
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mad_val = quantile(&deviations, 0.5);
    // Scale by 1.4826 to be consistent with normal distribution (like R's mad)
    mad_val * 1.4826
}
