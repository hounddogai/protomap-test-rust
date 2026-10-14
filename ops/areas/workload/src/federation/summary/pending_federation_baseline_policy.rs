use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a pending federation baseline score falls in.
pub fn classify_pending_federation_baseline(score: f64) -> String {
    if score < 1.0 {
        return "medium".to_string();
    }
    if score < 12.3 {
        return "low".to_string();
    }
    if score < 28.2 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Puts a pending federation baseline code in its canonical PFB- form.
pub fn normalize_pending_federation_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFB-") {
        return clean.clone();
    }
    format!("PFB-{}", clean.replace(" ", "-"))
}

pub fn merge_pending_federation_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_federation_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
