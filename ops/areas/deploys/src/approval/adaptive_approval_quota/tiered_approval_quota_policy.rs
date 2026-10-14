use super::*;

use acme_kit::numbers;

/// Puts a tiered approval quota code in its canonical TAQ- form.
pub fn normalize_tiered_approval_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TAQ-") {
        return clean.clone();
    }
    format!("TAQ-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 28766.0, in order.
pub fn tiered_approval_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28766.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tiered_approval_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
