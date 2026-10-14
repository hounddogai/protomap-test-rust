use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 19259.0, in order.
pub fn default_budget_burn_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19259.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a default budget burn metric code in its canonical DBBM- form.
pub fn normalize_default_budget_burn_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DBBM-") {
        return clean.clone();
    }
    format!("DBBM-{}", clean.replace(" ", "-"))
}

pub fn default_budget_burn_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.55;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
