use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a data export baseline code in its canonical DEB- form.
pub fn normalize_data_export_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEB-") {
        return clean.clone();
    }
    format!("DEB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 40473.0, in order.
pub fn data_export_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40473.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn data_export_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_data_export_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
