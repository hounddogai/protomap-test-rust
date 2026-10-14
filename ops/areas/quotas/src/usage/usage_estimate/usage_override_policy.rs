use super::*;

use acme_kit::numbers;

/// Puts a usage override code in its canonical UO- form.
pub fn normalize_usage_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("UO-") {
        return clean.clone();
    }
    format!("UO-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 2471.0, in order.
pub fn usage_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2471.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn usage_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
