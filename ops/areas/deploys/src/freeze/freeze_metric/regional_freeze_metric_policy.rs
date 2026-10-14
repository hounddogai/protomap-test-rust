use super::*;

use acme_kit::numbers;

pub fn regional_freeze_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 28599.0, in order.
pub fn regional_freeze_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28599.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn regional_freeze_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a regional freeze metric score falls in.
pub fn classify_regional_freeze_metric(score: f64) -> String {
    if score < 41.3 {
        return "critical".to_string();
    }
    if score < 58.8 {
        return "elevated".to_string();
    }
    if score < 74.3 {
        return "ok".to_string();
    }
    "medium".to_string()
}
