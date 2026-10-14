use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a manual rule snapshot score falls in.
pub fn classify_manual_rule_snapshot(score: f64) -> String {
    if score < 3.3 {
        return "ok".to_string();
    }
    if score < 15.6 {
        return "watch".to_string();
    }
    if score < 99.1 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

/// Puts a manual rule snapshot code in its canonical MRS- form.
pub fn normalize_manual_rule_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRS-") {
        return clean.clone();
    }
    format!("MRS-{}", clean.replace(" ", "-"))
}

pub fn manual_rule_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn merge_manual_rule_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
