//! Axis utilities for terminal-based plotting.
//!
//! Provides functions for generating nice axis breaks and formatting labels,
//! similar to ggplot2's pretty() function and Wilkinson's algorithm.

/// Calculate a "nice" number for axis labels using the "nice number" algorithm.
/// Based on ggplot2's pretty() function and Wilkinson's algorithm.
/// Returns a nice number that is a multiple of 1, 2, or 5.
///
/// # Arguments
/// * `x` - The input number to make nice
/// * `round` - If true, rounds to the nearest nice number; if false, finds the
///   nice number that is >= x (ceiling behavior)
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::nice_number;
///
/// assert_eq!(nice_number(23.0, true), 20.0);
/// assert_eq!(nice_number(23.0, false), 50.0);
/// assert_eq!(nice_number(0.0033, true), 0.005);
/// ```
pub fn nice_number(x: f64, round: bool) -> f64 {
    if x == 0.0 {
        return 0.0;
    }

    let exp = x.abs().log10().floor() as i32;
    let f = x / 10f64.powi(exp);

    let nf = if round {
        if f < 1.5 {
            1.0
        } else if f < 3.0 {
            2.0
        } else if f < 7.0 {
            5.0
        } else {
            10.0
        }
    } else {
        if f <= 1.0 {
            1.0
        } else if f <= 2.0 {
            2.0
        } else if f <= 5.0 {
            5.0
        } else {
            10.0
        }
    };

    nf * 10f64.powi(exp)
}

/// Generate nice axis breaks similar to ggplot2's pretty() function.
/// Returns a vector of nice tick positions.
///
/// # Arguments
/// * `min` - Minimum data value
/// * `max` - Maximum data value
/// * `n` - Desired number of ticks (will be approximated)
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::nice_breaks;
///
/// let breaks = nice_breaks(0.0, 50.0, 6);
/// // Returns something like [0.0, 10.0, 20.0, 30.0, 40.0, 50.0]
/// ```
pub fn nice_breaks(min: f64, max: f64, n: usize) -> Vec<f64> {
    if min == max {
        return vec![min];
    }

    let range = nice_number(max - min, false);
    let d = nice_number(range / (n as f64 - 1.0), true);
    let graph_min = (min / d).floor() * d;
    let graph_max = (max / d).ceil() * d;

    let mut breaks = Vec::new();
    let mut x = graph_min;
    while x <= graph_max {
        breaks.push(x);
        x += d;
    }

    breaks
}

/// Calculate the adaptive number of ticks based on chart dimension.
///
/// # Arguments
/// * `dimension` - The chart dimension in characters (width or height)
/// * `chars_per_tick` - Approximate characters needed per tick label
/// * `min_ticks` - Minimum number of ticks
/// * `max_ticks` - Maximum number of ticks
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::adaptive_tick_count;
///
/// // For an 80-character wide chart
/// let x_ticks = adaptive_tick_count(80, 15, 3, 8);
/// // Returns approximately 5 ticks
/// ```
pub fn adaptive_tick_count(
    dimension: usize,
    chars_per_tick: usize,
    min_ticks: usize,
    max_ticks: usize,
) -> usize {
    (dimension / chars_per_tick).max(min_ticks).min(max_ticks)
}

/// Determine appropriate precision for formatting a number based on its magnitude.
///
/// # Arguments
/// * `_value` - The value to format (unused, kept for API consistency)
/// * `range` - The data range (max - min)
///
/// # Returns
/// The number of decimal places to use (0-3)
pub fn format_precision(_value: f64, range: f64) -> usize {
    if range == 0.0 {
        return 2;
    }

    let magnitude = range.log10().floor() as i32;
    if magnitude >= 2 {
        0 // Whole numbers for large ranges
    } else if magnitude >= 0 {
        1 // 1 decimal place
    } else if magnitude >= -2 {
        2 // 2 decimal places
    } else {
        3 // 3 decimal places for very small ranges
    }
}

/// Format a number with appropriate precision and notation.
///
/// This function intelligently formats numbers based on their magnitude:
/// - Small integers (0-9999): No decimal places
/// - Large numbers (>= 10000 or < 0.001): Scientific notation
/// - Medium numbers: Appropriate decimal places based on precision
///
/// # Arguments
/// * `value` - The number to format
/// * `precision` - Number of decimal places for medium-range numbers
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::format_number;
///
/// assert_eq!(format_number(3.14159, 2), "3.14");
/// assert_eq!(format_number(42.0, 0), "42");
/// assert_eq!(format_number(12345.6, 2), "1.23e4");
/// assert_eq!(format_number(0.0001, 2), "1.00e-4");
/// ```
pub fn format_number(value: f64, precision: usize) -> String {
    // Handle special cases
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        };
    }
    if value == 0.0 {
        return "0".to_string();
    }

    let abs_value = value.abs();

    // Use scientific notation for very large or very small numbers
    if abs_value >= 10000.0 || abs_value < 0.001 {
        // Format with scientific notation, removing trailing zeros
        let formatted = format!("{:.*e}", precision, value);
        // Clean up: remove unnecessary + in exponent and trailing zeros
        formatted
            .replace("e+", "e")
            .replace("e0", "e")
            .replace("e-0", "e-")
    } else if abs_value >= 1.0 && precision == 0 {
        // Integer formatting for whole numbers
        format!("{:.0}", value)
    } else {
        // Standard decimal formatting
        let formatted = format!("{:.*}", precision, value);
        // Remove trailing zeros after decimal point
        if formatted.contains('.') {
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            formatted
        }
    }
}

/// Generate formatted axis labels for a given axis.
///
/// This is a convenience function that combines `nice_breaks`, `format_precision`,
/// and `format_number` to generate a complete set of axis labels.
/// It ensures that all labels are unique by increasing precision if needed.
///
/// # Arguments
/// * `min` - Minimum data value
/// * `max` - Maximum data value
/// * `dimension` - Chart dimension in characters (width for X, height for Y)
/// * `chars_per_tick` - Approximate characters per tick label
/// * `min_ticks` - Minimum number of ticks
/// * `max_ticks` - Maximum number of ticks
///
/// # Returns
/// A vector of formatted label strings
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::generate_axis_labels;
///
/// let labels = generate_axis_labels(0.0, 100.0, 80, 15, 3, 8);
/// // Returns something like ["0", "20", "40", "60", "80", "100"]
/// ```
pub fn generate_axis_labels(
    min: f64,
    max: f64,
    dimension: usize,
    chars_per_tick: usize,
    min_ticks: usize,
    max_ticks: usize,
) -> Vec<String> {
    let tick_count =
        adaptive_tick_count(dimension, chars_per_tick, min_ticks, max_ticks);
    let breaks = nice_breaks(min, max, tick_count);

    // Try to format labels, increasing precision if we get duplicates
    let mut precision = format_precision(min, max - min);
    let mut labels: Vec<String>;

    loop {
        labels = breaks
            .iter()
            .map(|&v| format_number(v, precision))
            .collect();

        // Check for duplicates
        let unique_labels: std::collections::HashSet<_> = labels.iter().collect();
        if unique_labels.len() == labels.len() || precision >= 6 {
            // No duplicates or we've reached max precision
            break;
        }

        // Increase precision and try again
        precision += 1;
    }

    labels
}

/// Generate formatted axis labels with fixed width for better visual alignment.
///
/// This function pads all labels to the same width using spaces, ensuring
/// that labels appear visually aligned when displayed.
///
/// # Arguments
/// * `min` - Minimum data value
/// * `max` - Maximum data value
/// * `dimension` - Chart dimension in characters (width for X, height for Y)
/// * `chars_per_tick` - Approximate characters per tick label
/// * `min_ticks` - Minimum number of ticks
/// * `max_ticks` - Maximum number of ticks
///
/// # Returns
/// A vector of formatted label strings with consistent width
pub fn generate_axis_labels_aligned(
    min: f64,
    max: f64,
    dimension: usize,
    chars_per_tick: usize,
    min_ticks: usize,
    max_ticks: usize,
) -> Vec<String> {
    let labels =
        generate_axis_labels(min, max, dimension, chars_per_tick, min_ticks, max_ticks);

    // Find the maximum width
    let max_width = labels.iter().map(|s| s.len()).max().unwrap_or(0);

    // Pad all labels to the same width (right-aligned)
    labels
        .into_iter()
        .map(|s| format!("{:>width$}", s, width = max_width))
        .collect()
}

/// Calculate axis bounds from data points with optional manual overrides.
///
/// This function computes the min/max bounds for X and Y axes from the provided data,
/// with support for manual override via xlim and ylim parameters.
///
/// # Arguments
/// * `data` - Iterator of (x, y) data points
/// * `xlim` - Optional manual X axis limits as (min, max)
/// * `ylim` - Optional manual Y axis limits as (min, max)
///
/// # Returns
/// A tuple of (x_min, x_max, y_min, y_max)
///
/// # Examples
/// ```
/// use tva::libs::plot::axis::calculate_bounds;
///
/// let data = vec![(1.0, 2.0), (3.0, 4.0), (5.0, 1.0)];
/// let (xmin, xmax, ymin, ymax) = calculate_bounds(data.iter().copied(), None, None);
/// ```
pub fn calculate_bounds(
    data: impl Iterator<Item = (f64, f64)>,
    xlim: Option<(f64, f64)>,
    ylim: Option<(f64, f64)>,
) -> (f64, f64, f64, f64) {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for (x, y) in data {
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }

    // Apply manual overrides
    if let Some((min, max)) = xlim {
        x_min = min;
        x_max = max;
    }

    if let Some((min, max)) = ylim {
        y_min = min;
        y_max = max;
    }

    // Ensure non-zero range
    if x_min == x_max {
        x_min -= 1.0;
        x_max += 1.0;
    }
    if y_min == y_max {
        y_min -= 1.0;
        y_max += 1.0;
    }

    (x_min, x_max, y_min, y_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_number() {
        assert_eq!(nice_number(23.0, true), 20.0);
        assert_eq!(nice_number(23.0, false), 50.0);
        assert_eq!(nice_number(0.0033, true), 0.005);
        assert_eq!(nice_number(0.0, true), 0.0);
    }

    #[test]
    fn test_nice_breaks() {
        let breaks = nice_breaks(0.0, 50.0, 6);
        assert_eq!(breaks, vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0]);

        let breaks = nice_breaks(0.001, 0.005, 5);
        assert!(breaks.len() >= 3);
        assert!(breaks[0] <= 0.001);
        assert!(breaks[breaks.len() - 1] >= 0.005);
    }

    #[test]
    fn test_adaptive_tick_count() {
        assert_eq!(adaptive_tick_count(80, 15, 3, 8), 5);
        assert_eq!(adaptive_tick_count(30, 15, 3, 8), 3);
        assert_eq!(adaptive_tick_count(200, 15, 3, 8), 8);
    }

    #[test]
    fn test_format_precision() {
        assert_eq!(format_precision(0.0, 100.0), 0);
        assert_eq!(format_precision(0.0, 10.0), 1);
        assert_eq!(format_precision(0.0, 0.1), 2);
        assert_eq!(format_precision(0.0, 0.001), 3);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(3.14159, 2), "3.14");
        assert_eq!(format_number(42.0, 0), "42");
        assert_eq!(format_number(0.001, 3), "0.001");
    }

    #[test]
    fn test_generate_axis_labels() {
        let labels = generate_axis_labels(0.0, 100.0, 80, 15, 3, 8);
        assert!(!labels.is_empty());
        assert_eq!(labels[0], "0");
        assert_eq!(labels[labels.len() - 1], "100");
    }
}
