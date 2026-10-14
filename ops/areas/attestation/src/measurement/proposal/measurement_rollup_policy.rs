use super::*;

use acme_kit::numbers;

/// Names the band a measurement rollup score falls in.
pub fn classify_measurement_rollup(score: f64) -> String {
    if score < 6.3 {
        return "medium".to_string();
    }
    if score < 24.2 {
        return "elevated".to_string();
    }
    if score < 39.5 {
        return "low".to_string();
    }
    if score < 64.9 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn measurement_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
