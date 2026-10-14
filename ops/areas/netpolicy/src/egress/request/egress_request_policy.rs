use super::*;

use acme_kit::numbers;

/// Puts a egress request code in its canonical ER- form.
pub fn normalize_egress_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ER-") {
        return clean.clone();
    }
    format!("ER-{}", clean.replace(" ", "-"))
}

pub fn egress_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 4505.0, in order.
pub fn egress_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4505.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
