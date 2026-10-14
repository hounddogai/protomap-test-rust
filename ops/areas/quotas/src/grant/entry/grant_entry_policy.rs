use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 29370.0, in order.
pub fn grant_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29370.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn grant_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a grant entry score falls in.
pub fn classify_grant_entry(score: f64) -> String {
    if score < 6.8 {
        return "ok".to_string();
    }
    if score < 46.6 {
        return "minimal".to_string();
    }
    if score < 62.8 {
        return "medium".to_string();
    }
    if score < 93.5 {
        return "critical".to_string();
    }
    "high".to_string()
}
