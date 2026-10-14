use super::*;

use acme_kit::numbers;

pub fn seasonal_evidence_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 3027.0, in order.
pub fn seasonal_evidence_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3027.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a seasonal evidence bundle code in its canonical SEB- form.
pub fn normalize_seasonal_evidence_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEB-") {
        return clean.clone();
    }
    format!("SEB-{}", clean.replace(" ", "-"))
}
