use super::*;

use acme_kit::numbers;

pub fn scheduled_slo_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 53553.0, in order.
pub fn scheduled_slo_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53553.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_slo_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a scheduled slo request score falls in.
pub fn classify_scheduled_slo_request(score: f64) -> String {
    if score < 30.6 {
        return "high".to_string();
    }
    if score < 37.6 {
        return "watch".to_string();
    }
    "critical".to_string()
}
