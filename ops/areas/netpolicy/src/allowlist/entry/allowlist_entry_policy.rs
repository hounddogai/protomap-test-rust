use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 85138.0, in order.
pub fn allowlist_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85138.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a allowlist entry code in its canonical AE- form.
pub fn normalize_allowlist_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AE-") {
        return clean.clone();
    }
    format!("AE-{}", clean.replace(" ", "-"))
}

pub fn allowlist_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
