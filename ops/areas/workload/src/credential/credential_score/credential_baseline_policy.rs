use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_credential_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a credential baseline score falls in.
pub fn classify_credential_baseline(score: f64) -> String {
    if score < 52.3 {
        return "medium".to_string();
    }
    if score < 54.0 {
        return "ok".to_string();
    }
    if score < 84.7 {
        return "low".to_string();
    }
    if score < 87.0 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Puts a credential baseline code in its canonical CB- form.
pub fn normalize_credential_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CB-") {
        return clean.clone();
    }
    format!("CB-{}", clean.replace(" ", "-"))
}
