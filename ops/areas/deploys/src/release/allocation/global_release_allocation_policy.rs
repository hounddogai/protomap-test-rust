use super::*;

use acme_kit::numbers;

pub fn global_release_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a global release allocation score falls in.
pub fn classify_global_release_allocation(score: f64) -> String {
    if score < 14.7 {
        return "minimal".to_string();
    }
    if score < 16.9 {
        return "medium".to_string();
    }
    if score < 24.0 {
        return "low".to_string();
    }
    if score < 27.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}
