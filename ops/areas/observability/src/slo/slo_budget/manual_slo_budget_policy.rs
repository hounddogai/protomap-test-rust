use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a manual slo budget score falls in.
pub fn classify_manual_slo_budget(score: f64) -> String {
    if score < 13.6 {
        return "watch".to_string();
    }
    if score < 19.1 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a manual slo budget code in its canonical MSB- form.
pub fn normalize_manual_slo_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSB-") {
        return clean.clone();
    }
    format!("MSB-{}", clean.replace(" ", "-"))
}

pub fn manual_slo_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_manual_slo_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
