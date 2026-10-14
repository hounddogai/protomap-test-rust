use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a escalation template score falls in.
pub fn classify_escalation_template(score: f64) -> String {
    if score < 21.5 {
        return "ok".to_string();
    }
    if score < 33.6 {
        return "high".to_string();
    }
    if score < 34.1 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn escalation_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_escalation_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
