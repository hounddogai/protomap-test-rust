use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn legacy_evidence_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_legacy_evidence_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy evidence template score falls in.
pub fn classify_legacy_evidence_template(score: f64) -> String {
    if score < 21.8 {
        return "critical".to_string();
    }
    if score < 82.4 {
        return "low".to_string();
    }
    if score < 83.3 {
        return "high".to_string();
    }
    "ok".to_string()
}
