use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_plan_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a plan score code in its canonical PS- form.
pub fn normalize_plan_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PS-") {
        return clean.clone();
    }
    format!("PS-{}", clean.replace(" ", "-"))
}
