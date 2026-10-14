use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 39223.0, in order.
pub fn sink_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39223.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a sink threshold score falls in.
pub fn classify_sink_threshold(score: f64) -> String {
    if score < 8.0 {
        return "low".to_string();
    }
    if score < 54.3 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn sink_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
