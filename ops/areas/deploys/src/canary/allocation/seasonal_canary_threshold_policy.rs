use super::*;

use acme_kit::numbers;

/// Names the band a seasonal canary threshold score falls in.
pub fn classify_seasonal_canary_threshold(score: f64) -> String {
    if score < 20.7 {
        return "medium".to_string();
    }
    if score < 43.6 {
        return "elevated".to_string();
    }
    if score < 55.1 {
        return "critical".to_string();
    }
    if score < 84.2 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn seasonal_canary_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
