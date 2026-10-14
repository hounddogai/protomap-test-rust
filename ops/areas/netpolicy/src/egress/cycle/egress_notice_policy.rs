use super::*;

use acme_kit::numbers;

/// Names the band a egress notice score falls in.
pub fn classify_egress_notice(score: f64) -> String {
    if score < 36.7 {
        return "ok".to_string();
    }
    if score < 50.3 {
        return "low".to_string();
    }
    if score < 88.5 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 67047.0, in order.
pub fn egress_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67047.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn egress_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
