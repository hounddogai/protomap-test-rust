use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_manual_tenant_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual tenant notice score falls in.
pub fn classify_manual_tenant_notice(score: f64) -> String {
    if score < 7.7 {
        return "watch".to_string();
    }
    if score < 8.1 {
        return "elevated".to_string();
    }
    if score < 49.8 {
        return "high".to_string();
    }
    if score < 68.6 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn manual_tenant_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn manual_tenant_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
