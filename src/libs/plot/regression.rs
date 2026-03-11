//! Linear regression utilities for plotting.

/// Calculate linear regression (least squares) for a set of points.
/// Returns (slope, intercept) for the line y = slope * x + intercept
pub fn calculate_regression(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if n < 2.0 {
        return None;
    }

    let sum_x: f64 = points.iter().map(|p| p.0).sum();
    let sum_y: f64 = points.iter().map(|p| p.1).sum();
    let sum_xy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let sum_x2: f64 = points.iter().map(|p| p.0 * p.0).sum();

    let denominator = n * sum_x2 - sum_x * sum_x;
    if denominator.abs() < f64::EPSILON {
        return None; // Vertical line or no variation in x
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;

    Some((slope, intercept))
}

/// Format regression equation for display in legend.
/// Shows y = mx + b with appropriate precision.
pub fn format_regression_equation(slope: f64, intercept: f64) -> String {
    // Determine appropriate precision based on magnitude
    let format_coefficient = |v: f64| -> String {
        if v.abs() >= 100.0 {
            format!("{:.0}", v)
        } else if v.abs() >= 10.0 {
            format!("{:.1}", v)
        } else if v.abs() >= 1.0 {
            format!("{:.2}", v)
        } else if v.abs() >= 0.1 {
            format!("{:.3}", v)
        } else {
            format!("{:.4}", v)
        }
    };

    let slope_str = format_coefficient(slope);
    let intercept_str = format_coefficient(intercept);

    if intercept >= 0.0 {
        format!("y = {}x + {}", slope_str, intercept_str)
    } else {
        format!(
            "y = {}x - {}",
            slope_str,
            format_coefficient(intercept.abs())
        )
    }
}

/// Generate regression line points for plotting.
/// Returns two points at the x-axis boundaries.
pub fn generate_regression_points(
    slope: f64,
    intercept: f64,
    x_min: f64,
    x_max: f64,
) -> Vec<(f64, f64)> {
    vec![
        (x_min, slope * x_min + intercept),
        (x_max, slope * x_max + intercept),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_regression_linear() {
        // y = 2x
        let points = vec![(0.0, 0.0), (1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let (slope, intercept) = calculate_regression(&points).unwrap();
        assert!((slope - 2.0).abs() < 0.001);
        assert!(intercept.abs() < 0.001);
    }

    #[test]
    fn test_calculate_regression_with_intercept() {
        // y = 0.5x + 1
        let points = vec![(0.0, 1.0), (2.0, 2.0), (4.0, 3.0)];
        let (slope, intercept) = calculate_regression(&points).unwrap();
        assert!((slope - 0.5).abs() < 0.001);
        assert!((intercept - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_regression_insufficient_points() {
        let points = vec![(1.0, 1.0)];
        assert!(calculate_regression(&points).is_none());
    }

    #[test]
    fn test_format_regression_equation_positive() {
        assert_eq!(format_regression_equation(0.5, 1.0), "y = 0.500x + 1.00");
    }

    #[test]
    fn test_format_regression_equation_negative() {
        assert_eq!(format_regression_equation(0.5, -1.0), "y = 0.500x - 1.00");
    }

    #[test]
    fn test_format_regression_equation_large_values() {
        assert_eq!(format_regression_equation(150.5, 200.3), "y = 150x + 200");
    }

    #[test]
    fn test_generate_regression_points() {
        let points = generate_regression_points(2.0, 0.0, 0.0, 10.0);
        assert_eq!(points, vec![(0.0, 0.0), (10.0, 20.0)]);
    }
}
