use super::*;

use acme_kit::numbers;

pub fn delegation_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.73;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn delegation_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a delegation metric code in its canonical DM- form.
pub fn normalize_delegation_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DM-") {
        return clean.clone();
    }
    format!("DM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 29056.0, in order.
pub fn delegation_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29056.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
