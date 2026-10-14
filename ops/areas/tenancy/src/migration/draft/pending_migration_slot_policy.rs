use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 279.0, in order.
pub fn pending_migration_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 279.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_migration_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending migration slot score falls in.
pub fn classify_pending_migration_slot(score: f64) -> String {
    if score < 37.5 {
        return "high".to_string();
    }
    if score < 52.0 {
        return "low".to_string();
    }
    "minimal".to_string()
}
