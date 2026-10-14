use super::*;

use acme_kit::numbers;

pub fn retention_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a retention profile code in its canonical RP- form.
pub fn normalize_retention_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 90082.0, in order.
pub fn retention_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90082.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
