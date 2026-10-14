use super::*;

use acme_kit::numbers;

/// Names the band a adaptive region pin review score falls in.
pub fn classify_adaptive_region_pin_review(score: f64) -> String {
    if score < 25.0 {
        return "low".to_string();
    }
    if score < 77.1 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn adaptive_region_pin_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
