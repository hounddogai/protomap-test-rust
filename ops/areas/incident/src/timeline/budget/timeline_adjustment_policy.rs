use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a timeline adjustment code in its canonical TA- form.
pub fn normalize_timeline_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TA-") {
        return clean.clone();
    }
    format!("TA-{}", clean.replace(" ", "-"))
}

pub fn merge_timeline_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
