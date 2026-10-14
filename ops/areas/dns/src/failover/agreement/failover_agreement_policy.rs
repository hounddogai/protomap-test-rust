use super::*;

use acme_kit::numbers;

pub fn failover_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a failover agreement code in its canonical FA- form.
pub fn normalize_failover_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 93806.0, in order.
pub fn failover_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93806.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
