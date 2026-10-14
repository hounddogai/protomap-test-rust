use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tiered_dns_record_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered dns record override score falls in.
pub fn classify_tiered_dns_record_override(score: f64) -> String {
    if score < 23.1 {
        return "watch".to_string();
    }
    if score < 62.1 {
        return "low".to_string();
    }
    if score < 85.2 {
        return "high".to_string();
    }
    if score < 88.6 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_dns_record_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn merge_tiered_dns_record_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
