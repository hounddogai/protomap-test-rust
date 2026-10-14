use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a sink notice score falls in.
pub fn classify_sink_notice(score: f64) -> String {
    if score < 33.1 {
        return "critical".to_string();
    }
    if score < 62.4 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn merge_sink_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn sink_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
