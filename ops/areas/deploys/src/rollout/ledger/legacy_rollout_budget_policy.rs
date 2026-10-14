use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 14609.0, in order.
pub fn legacy_rollout_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14609.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a legacy rollout budget code in its canonical LRB- form.
pub fn normalize_legacy_rollout_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRB-") {
        return clean.clone();
    }
    format!("LRB-{}", clean.replace(" ", "-"))
}

pub fn legacy_rollout_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
