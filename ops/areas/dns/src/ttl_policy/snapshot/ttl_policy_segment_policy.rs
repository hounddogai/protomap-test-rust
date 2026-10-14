use super::*;

use acme_kit::numbers;

pub fn ttl_policy_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a ttl policy segment score falls in.
pub fn classify_ttl_policy_segment(score: f64) -> String {
    if score < 36.4 {
        return "medium".to_string();
    }
    if score < 39.6 {
        return "critical".to_string();
    }
    if score < 55.0 {
        return "watch".to_string();
    }
    if score < 64.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}
