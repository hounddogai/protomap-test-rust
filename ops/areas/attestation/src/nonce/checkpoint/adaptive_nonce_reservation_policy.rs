use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 64781.0, in order.
pub fn adaptive_nonce_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64781.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_nonce_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a adaptive nonce reservation code in its canonical ANR- form.
pub fn normalize_adaptive_nonce_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ANR-") {
        return clean.clone();
    }
    format!("ANR-{}", clean.replace(" ", "-"))
}
