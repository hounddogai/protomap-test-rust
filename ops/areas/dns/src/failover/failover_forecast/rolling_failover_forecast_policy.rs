use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_rolling_failover_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a rolling failover forecast code in its canonical RFF- form.
pub fn normalize_rolling_failover_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFF-") {
        return clean.clone();
    }
    format!("RFF-{}", clean.replace(" ", "-"))
}
