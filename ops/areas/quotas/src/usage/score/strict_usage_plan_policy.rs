use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a strict usage plan code in its canonical SUP- form.
pub fn normalize_strict_usage_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SUP-") {
        return clean.clone();
    }
    format!("SUP-{}", clean.replace(" ", "-"))
}

pub fn merge_strict_usage_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
