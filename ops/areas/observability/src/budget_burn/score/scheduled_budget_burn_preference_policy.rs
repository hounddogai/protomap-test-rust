use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_scheduled_budget_burn_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a scheduled budget burn preference code in its canonical SBBP- form.
pub fn normalize_scheduled_budget_burn_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBBP-") {
        return clean.clone();
    }
    format!("SBBP-{}", clean.replace(" ", "-"))
}
