use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 62849.0, in order.
pub fn overage_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62849.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a overage budget code in its canonical OB- form.
pub fn normalize_overage_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OB-") {
        return clean.clone();
    }
    format!("OB-{}", clean.replace(" ", "-"))
}

pub fn overage_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
