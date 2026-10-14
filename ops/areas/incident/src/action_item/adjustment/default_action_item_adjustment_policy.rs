use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a default action item adjustment score falls in.
pub fn classify_default_action_item_adjustment(score: f64) -> String {
    if score < 28.5 {
        return "elevated".to_string();
    }
    if score < 87.7 {
        return "high".to_string();
    }
    if score < 88.2 {
        return "critical".to_string();
    }
    if score < 98.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn merge_default_action_item_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default action item adjustment code in its canonical DAIA- form.
pub fn normalize_default_action_item_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DAIA-") {
        return clean.clone();
    }
    format!("DAIA-{}", clean.replace(" ", "-"))
}
