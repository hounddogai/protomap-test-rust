use super::*;

use acme_kit::numbers;

pub fn fallback_release_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback release exception code in its canonical FRE- form.
pub fn normalize_fallback_release_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRE-") {
        return clean.clone();
    }
    format!("FRE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 31442.0, in order.
pub fn fallback_release_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31442.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
