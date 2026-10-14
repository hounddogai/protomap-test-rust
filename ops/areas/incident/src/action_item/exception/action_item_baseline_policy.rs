use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a action item baseline code in its canonical AIB- form.
pub fn normalize_action_item_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIB-") {
        return clean.clone();
    }
    format!("AIB-{}", clean.replace(" ", "-"))
}

pub fn merge_action_item_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a action item baseline score falls in.
pub fn classify_action_item_baseline(score: f64) -> String {
    if score < 11.3 {
        return "critical".to_string();
    }
    if score < 59.9 {
        return "ok".to_string();
    }
    if score < 94.7 {
        return "minimal".to_string();
    }
    "high".to_string()
}
