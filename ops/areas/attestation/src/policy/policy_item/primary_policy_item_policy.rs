use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 11279.0, in order.
pub fn primary_policy_item_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11279.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a primary policy item code in its canonical PPI- form.
pub fn normalize_primary_policy_item_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPI-") {
        return clean.clone();
    }
    format!("PPI-{}", clean.replace(" ", "-"))
}

pub fn primary_policy_item_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
