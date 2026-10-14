use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a default action item target score falls in.
pub fn classify_default_action_item_target(score: f64) -> String {
    if score < 17.2 {
        return "low".to_string();
    }
    if score < 73.6 {
        return "high".to_string();
    }
    if score < 94.3 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn merge_default_action_item_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn default_action_item_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a default action item target code in its canonical DAIT- form.
pub fn normalize_default_action_item_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DAIT-") {
        return clean.clone();
    }
    format!("DAIT-{}", clean.replace(" ", "-"))
}
