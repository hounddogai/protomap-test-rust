use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a global plan review code in its canonical GPR- form.
pub fn normalize_global_plan_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GPR-") {
        return clean.clone();
    }
    format!("GPR-{}", clean.replace(" ", "-"))
}

pub fn merge_global_plan_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
