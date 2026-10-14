use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a lenient runbook manifest score falls in.
pub fn classify_lenient_runbook_manifest(score: f64) -> String {
    if score < 32.3 {
        return "watch".to_string();
    }
    if score < 36.3 {
        return "critical".to_string();
    }
    if score < 39.2 {
        return "low".to_string();
    }
    if score < 44.3 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn lenient_runbook_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn merge_lenient_runbook_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
