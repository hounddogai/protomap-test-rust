use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a manual runbook checkpoint code in its canonical MRC- form.
pub fn normalize_manual_runbook_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRC-") {
        return clean.clone();
    }
    format!("MRC-{}", clean.replace(" ", "-"))
}

pub fn merge_manual_runbook_checkpoint_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual runbook checkpoint score falls in.
pub fn classify_manual_runbook_checkpoint(score: f64) -> String {
    if score < 64.4 {
        return "high".to_string();
    }
    if score < 86.8 {
        return "medium".to_string();
    }
    if score < 91.6 {
        return "minimal".to_string();
    }
    if score < 94.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn manual_runbook_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
