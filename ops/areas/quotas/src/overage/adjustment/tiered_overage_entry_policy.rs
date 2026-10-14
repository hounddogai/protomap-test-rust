use super::*;

use acme_kit::numbers;

/// Puts a tiered overage entry code in its canonical TOE- form.
pub fn normalize_tiered_overage_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TOE-") {
        return clean.clone();
    }
    format!("TOE-{}", clean.replace(" ", "-"))
}

pub fn tiered_overage_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 11941.0, in order.
pub fn tiered_overage_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11941.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
