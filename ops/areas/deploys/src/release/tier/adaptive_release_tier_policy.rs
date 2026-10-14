use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_adaptive_release_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive release tier score falls in.
pub fn classify_adaptive_release_tier(score: f64) -> String {
    if score < 17.4 {
        return "minimal".to_string();
    }
    if score < 42.2 {
        return "watch".to_string();
    }
    if score < 56.9 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn adaptive_release_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
