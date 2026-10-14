use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_global_alert_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a global alert cycle score falls in.
pub fn classify_global_alert_cycle(score: f64) -> String {
    if score < 69.5 {
        return "elevated".to_string();
    }
    if score < 77.2 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Puts a global alert cycle code in its canonical GAC- form.
pub fn normalize_global_alert_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GAC-") {
        return clean.clone();
    }
    format!("GAC-{}", clean.replace(" ", "-"))
}
