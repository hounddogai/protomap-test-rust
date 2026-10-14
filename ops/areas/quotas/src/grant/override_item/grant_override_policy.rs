use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 80773.0, in order.
pub fn grant_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80773.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn grant_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a grant override code in its canonical GO- form.
pub fn normalize_grant_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GO-") {
        return clean.clone();
    }
    format!("GO-{}", clean.replace(" ", "-"))
}

pub fn grant_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
