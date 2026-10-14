use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_legacy_isolation_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a legacy isolation ledger code in its canonical LIL- form.
pub fn normalize_legacy_isolation_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LIL-") {
        return clean.clone();
    }
    format!("LIL-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy isolation ledger score falls in.
pub fn classify_legacy_isolation_ledger(score: f64) -> String {
    if score < 18.3 {
        return "medium".to_string();
    }
    if score < 19.1 {
        return "low".to_string();
    }
    "critical".to_string()
}
