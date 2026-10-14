use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 35605.0, in order.
pub fn retention_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35605.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn retention_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a retention snapshot code in its canonical RS- form.
pub fn normalize_retention_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RS-") {
        return clean.clone();
    }
    format!("RS-{}", clean.replace(" ", "-"))
}

pub fn retention_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
