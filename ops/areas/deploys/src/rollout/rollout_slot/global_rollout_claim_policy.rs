use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 36347.0, in order.
pub fn global_rollout_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36347.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_rollout_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn global_rollout_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a global rollout claim code in its canonical GRC- form.
pub fn normalize_global_rollout_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRC-") {
        return clean.clone();
    }
    format!("GRC-{}", clean.replace(" ", "-"))
}
