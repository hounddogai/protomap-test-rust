use super::*;

use acme_kit::numbers;

pub fn binding_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a binding quota score falls in.
pub fn classify_binding_quota(score: f64) -> String {
    if score < 16.9 {
        return "watch".to_string();
    }
    if score < 18.4 {
        return "high".to_string();
    }
    if score < 72.5 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
