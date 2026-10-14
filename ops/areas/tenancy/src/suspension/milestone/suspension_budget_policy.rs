use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn suspension_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a suspension budget code in its canonical SB- form.
pub fn normalize_suspension_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SB-") {
        return clean.clone();
    }
    format!("SB-{}", clean.replace(" ", "-"))
}

pub fn merge_suspension_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
