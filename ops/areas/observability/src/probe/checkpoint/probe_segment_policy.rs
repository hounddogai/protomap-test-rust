use super::*;

use acme_kit::numbers;

pub fn probe_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a probe segment score falls in.
pub fn classify_probe_segment(score: f64) -> String {
    if score < 32.6 {
        return "medium".to_string();
    }
    if score < 33.7 {
        return "critical".to_string();
    }
    if score < 45.7 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
