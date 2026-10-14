use super::*;

use acme_kit::numbers;

/// Names the band a pending runbook draft score falls in.
pub fn classify_pending_runbook_draft(score: f64) -> String {
    if score < 51.5 {
        return "elevated".to_string();
    }
    if score < 59.8 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 99502.0, in order.
pub fn pending_runbook_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99502.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_runbook_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
