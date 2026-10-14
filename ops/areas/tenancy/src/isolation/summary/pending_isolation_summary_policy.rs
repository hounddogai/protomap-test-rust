use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a pending isolation summary score falls in.
pub fn classify_pending_isolation_summary(score: f64) -> String {
    if score < 1.9 {
        return "minimal".to_string();
    }
    if score < 62.6 {
        return "low".to_string();
    }
    if score < 70.8 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn merge_pending_isolation_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a pending isolation summary code in its canonical PIS- form.
pub fn normalize_pending_isolation_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PIS-") {
        return clean.clone();
    }
    format!("PIS-{}", clean.replace(" ", "-"))
}
