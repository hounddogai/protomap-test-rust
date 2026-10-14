use super::*;

use acme_kit::numbers;

pub fn probe_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a probe review score falls in.
pub fn classify_probe_review(score: f64) -> String {
    if score < 36.9 {
        return "minimal".to_string();
    }
    if score < 69.5 {
        return "high".to_string();
    }
    "elevated".to_string()
}
