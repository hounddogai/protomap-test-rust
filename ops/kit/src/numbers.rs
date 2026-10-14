//! Arithmetic helpers: clamping, rounding, and simple statistics.

pub fn clamp(value: i64, low: i64, high: i64) -> i64 {
    if value < low {
        return low;
    }
    if value > high {
        return high;
    }
    value
}

pub fn clamp_float(value: f64, low: f64, high: f64) -> f64 {
    value.max(low).min(high)
}

/// Returns part as a percentage of whole, rounded to two places, and 0 when whole is 0.
pub fn percent(part: f64, whole: f64) -> f64 {
    if whole == 0.0 {
        return 0.0;
    }
    (part / whole * 100.0 * 100.0).round() / 100.0
}

pub fn round_to(value: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return value;
    }
    (value / step + 0.5).floor() * step
}

pub fn mean(values: &[f64]) -> f64 {
    if (values.len() as i64) == 0 {
        return 0.0;
    }
    let mut total = 0.0;
    for value in values.iter().copied() {
        total += value;
    }
    total / ((values.len() as i64) as f64)
}

/// Returns the values in ascending order, leaving the input as it was.
pub fn sorted_floats(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    for i in 0..sorted.len() as i64 {
        for k in 0..(sorted.len() as i64) - i - 1 {
            if sorted[k as usize] > sorted[(k + 1) as usize] {
                let swap = sorted[k as usize];
                sorted[k as usize] = sorted[(k + 1) as usize];
                sorted[(k + 1) as usize] = swap;
            }
        }
    }
    sorted
}

/// Returns the p-th percentile by the nearest-rank method.
pub fn percentile(values: &[f64], p: f64) -> f64 {
    if (values.len() as i64) == 0 {
        return 0.0;
    }
    let sorted = sorted_floats(values);
    let rank = (p / 100.0 * (((sorted.len() as i64) - 1) as f64)) as i64;
    sorted[clamp(rank, 0, (sorted.len() as i64) - 1) as usize]
}

pub fn weighted_mean(values: &[f64], weights: &[f64]) -> Result<f64, String> {
    if (values.len() as i64) != (weights.len() as i64) {
        return Err(format!(
            "got {} values but {} weights",
            values.len() as i64,
            weights.len() as i64
        ));
    }
    let mut sum = 0.0;
    let mut weight = 0.0;
    for i in 0..values.len() as i64 {
        sum += values[i as usize] * weights[i as usize];
        weight += weights[i as usize];
    }
    if weight == 0.0 {
        return Ok(0.0);
    }
    Ok(sum / weight)
}

pub fn growth_rate(previous: f64, current: f64) -> f64 {
    if previous == 0.0 {
        return 0.0;
    }
    ((current - previous) / previous.abs() * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_into_range() {
        assert_eq!(clamp(12, 0, 10), 10, "high");
        assert_eq!(clamp(-3, 0, 10), 0, "low");
        assert_eq!(percent(1.0, 4.0), 25.0, "percent");
    }

    #[test]
    fn finds_the_median_by_rank() {
        assert_eq!(percentile(&vec![5.0, 1.0, 3.0], 50.0), 3.0, "median");
        assert_eq!(mean(&[]), 0.0, "empty mean");
        assert!(
            weighted_mean(&vec![1.0], &[]).is_err(),
            "expected mismatched weights to fail"
        );
    }
}
