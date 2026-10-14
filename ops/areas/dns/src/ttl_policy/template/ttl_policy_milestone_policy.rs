use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn ttl_policy_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a ttl policy milestone score falls in.
pub fn classify_ttl_policy_milestone(score: f64) -> String {
    if score < 19.1 {
        return "low".to_string();
    }
    if score < 54.9 {
        return "high".to_string();
    }
    if score < 55.1 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn ttl_policy_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_ttl_policy_milestone_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
