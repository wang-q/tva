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

/// Calculates the range (max - min).
#[inline]
pub fn range(min: f64, max: f64) -> f64 {
    if min.is_finite() && max.is_finite() {
        max - min
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

    let mut deviations: Vec<f64> =
        sorted_vals.iter().map(|v| (v - median).abs()).collect();
    // We need to sort deviations to find their median
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mad_val = quantile(&deviations, 0.5);
    // Scale by 1.4826 to be consistent with normal distribution (like R's mad)
    mad_val * 1.4826
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert_eq!(mean(10.0, 2), 5.0);
        assert!(mean(0.0, 0).is_nan());
    }

    #[test]
    fn test_geomean() {
        // ln(2) + ln(8) = 0.693 + 2.079 = 2.772
        // geomean = exp(2.772 / 2) = exp(1.386) = 4.0
        let sum_log = 2.0_f64.ln() + 8.0_f64.ln();
        let res = geomean(sum_log, 2);
        assert!((res - 4.0).abs() < 1e-10);

        assert!(geomean(0.0, 0).is_nan());
    }

    #[test]
    fn test_harmmean() {
        // 1/2 + 1/4 = 0.75
        // harmmean = 2 / 0.75 = 2.666
        let sum_inv = 0.5 + 0.25;
        let res = harmmean(sum_inv, 2);
        assert!((res - 2.6666666666666665).abs() < 1e-10);

        assert!(harmmean(0.0, 0).is_nan());
        assert!(harmmean(0.0, 2).is_nan()); // sum_inv = 0
    }

    #[test]
    fn test_variance() {
        // 2, 4
        // mean = 3
        // sum = 6
        // sum_sq = 4 + 16 = 20
        // var = (20 - 6*3) / (2-1) = 20 - 18 = 2
        let res = variance(20.0, 6.0, 2);
        assert!((res - 2.0).abs() < 1e-10);

        assert!(variance(0.0, 0.0, 1).is_nan());
    }

    #[test]
    fn test_stdev() {
        // 2, 4 -> var = 2 -> stdev = sqrt(2) = 1.414
        let res = stdev(20.0, 6.0, 2);
        assert!((res - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_cv() {
        // 2, 4 -> mean = 3, stdev = 1.414
        // cv = 1.414 / 3 = 0.4714
        let res = cv(20.0, 6.0, 2);
        assert!((res - (2.0_f64.sqrt() / 3.0)).abs() < 1e-10);

        assert!(cv(0.0, 0.0, 1).is_nan());
        assert!(cv(0.0, 0.0, 2).is_nan()); // mean = 0
    }

    #[test]
    fn test_range() {
        assert_eq!(range(2.0, 10.0), 8.0);
        assert!(range(f64::NAN, 10.0).is_nan());
    }

    #[test]
    fn test_quantile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // Median
        assert_eq!(quantile(&data, 0.5), 3.0);
        // Min
        assert_eq!(quantile(&data, 0.0), 1.0);
        // Max
        assert_eq!(quantile(&data, 1.0), 5.0);

        // Interpolation
        // 1, 2, 3, 4
        let data2 = vec![1.0, 2.0, 3.0, 4.0];
        // 0.5 * (3) = 1.5 -> index 1 (2.0) and 2 (3.0) -> 2.5
        assert_eq!(quantile(&data2, 0.5), 2.5);

        assert!(quantile(&[], 0.5).is_nan());
        assert_eq!(quantile(&[1.0], 0.5), 1.0);
    }

    #[test]
    fn test_mad() {
        // 1, 2, 3, 4, 5 -> median = 3
        // devs: 2, 1, 0, 1, 2 -> sort -> 0, 1, 1, 2, 2
        // median dev = 1
        // mad = 1 * 1.4826
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let res = mad(&data);
        assert!((res - 1.4826).abs() < 1e-10);

        assert!(mad(&[]).is_nan());
    }
}
