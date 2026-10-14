use super::*;

use acme_kit::numbers;

/// Names the band a retention segment score falls in.
pub fn classify_retention_segment(score: f64) -> String {
    if score < 11.7 {
        return "low".to_string();
    }
    if score < 32.4 {
        return "minimal".to_string();
    }
    if score < 95.1 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn retention_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
