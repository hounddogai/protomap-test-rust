use super::*;

use acme_kit::numbers;

/// Names the band a zone snapshot score falls in.
pub fn classify_zone_snapshot(score: f64) -> String {
    if score < 6.0 {
        return "ok".to_string();
    }
    if score < 9.1 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn zone_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.04;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 72096.0, in order.
pub fn zone_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72096.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
