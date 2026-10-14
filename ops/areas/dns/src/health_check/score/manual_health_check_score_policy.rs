use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_health_check_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_health_check_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a manual health check score score falls in.
pub fn classify_manual_health_check_score(score: f64) -> String {
    if score < 30.1 {
        return "low".to_string();
    }
    if score < 39.4 {
        return "minimal".to_string();
    }
    if score < 91.2 {
        return "high".to_string();
    }
    "medium".to_string()
}
