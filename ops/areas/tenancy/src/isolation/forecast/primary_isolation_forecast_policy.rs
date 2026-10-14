use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a primary isolation forecast code in its canonical PIF- form.
pub fn normalize_primary_isolation_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PIF-") {
        return clean.clone();
    }
    format!("PIF-{}", clean.replace(" ", "-"))
}

pub fn merge_primary_isolation_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
