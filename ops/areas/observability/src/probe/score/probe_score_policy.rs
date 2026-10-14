use super::*;

use acme_kit::numbers;

/// Puts a probe score code in its canonical PS- form.
pub fn normalize_probe_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PS-") {
        return clean.clone();
    }
    format!("PS-{}", clean.replace(" ", "-"))
}

pub fn probe_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.39;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 50236.0, in order.
pub fn probe_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50236.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
