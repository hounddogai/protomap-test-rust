use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 40056.0, in order.
pub fn adaptive_runbook_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40056.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_runbook_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a adaptive runbook bundle code in its canonical ARB- form.
pub fn normalize_adaptive_runbook_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARB-") {
        return clean.clone();
    }
    format!("ARB-{}", clean.replace(" ", "-"))
}
