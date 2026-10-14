use super::*;

use acme_kit::numbers;

pub fn manual_release_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a manual release mapping code in its canonical MRM- form.
pub fn normalize_manual_release_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRM-") {
        return clean.clone();
    }
    format!("MRM-{}", clean.replace(" ", "-"))
}

pub fn manual_release_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 31272.0, in order.
pub fn manual_release_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31272.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
