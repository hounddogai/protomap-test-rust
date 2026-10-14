use super::*;

use acme_kit::numbers;

pub fn manual_ttl_policy_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a manual ttl policy threshold code in its canonical MTPT- form.
pub fn normalize_manual_ttl_policy_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MTPT-") {
        return clean.clone();
    }
    format!("MTPT-{}", clean.replace(" ", "-"))
}

pub fn manual_ttl_policy_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 66052.0, in order.
pub fn manual_ttl_policy_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66052.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
