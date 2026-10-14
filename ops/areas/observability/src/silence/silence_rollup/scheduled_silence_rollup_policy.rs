use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a scheduled silence rollup score falls in.
pub fn classify_scheduled_silence_rollup(score: f64) -> String {
    if score < 33.0 {
        return "ok".to_string();
    }
    if score < 59.6 {
        return "critical".to_string();
    }
    if score < 73.1 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn merge_scheduled_silence_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
