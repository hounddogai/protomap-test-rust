use super::*;

use acme_kit::numbers;

/// Names the band a rolling approval variance score falls in.
pub fn classify_rolling_approval_variance(score: f64) -> String {
    if score < 50.3 {
        return "critical".to_string();
    }
    if score < 72.7 {
        return "high".to_string();
    }
    if score < 91.6 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn rolling_approval_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
