use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 27012.0, in order.
pub fn adaptive_ttl_policy_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 27012.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_ttl_policy_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn adaptive_ttl_policy_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a adaptive ttl policy snapshot code in its canonical ATPS- form.
pub fn normalize_adaptive_ttl_policy_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ATPS-") {
        return clean.clone();
    }
    format!("ATPS-{}", clean.replace(" ", "-"))
}
