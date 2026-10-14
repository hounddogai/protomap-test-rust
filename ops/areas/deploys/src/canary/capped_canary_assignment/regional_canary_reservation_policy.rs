use super::*;

use acme_kit::numbers;

/// Puts a regional canary reservation code in its canonical RCR- form.
pub fn normalize_regional_canary_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCR-") {
        return clean.clone();
    }
    format!("RCR-{}", clean.replace(" ", "-"))
}

pub fn regional_canary_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn regional_canary_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 52761.0, in order.
pub fn regional_canary_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52761.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
