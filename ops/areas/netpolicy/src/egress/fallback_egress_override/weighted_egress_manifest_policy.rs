use super::*;

use acme_kit::numbers;

pub fn weighted_egress_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 69725.0, in order.
pub fn weighted_egress_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69725.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn weighted_egress_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a weighted egress manifest code in its canonical WEM- form.
pub fn normalize_weighted_egress_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WEM-") {
        return clean.clone();
    }
    format!("WEM-{}", clean.replace(" ", "-"))
}
