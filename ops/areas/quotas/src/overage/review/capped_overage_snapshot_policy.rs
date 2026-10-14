use super::*;

use acme_kit::numbers;

pub fn capped_overage_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a capped overage snapshot code in its canonical COS- form.
pub fn normalize_capped_overage_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("COS-") {
        return clean.clone();
    }
    format!("COS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 20772.0, in order.
pub fn capped_overage_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20772.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
