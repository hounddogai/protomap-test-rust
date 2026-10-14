use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 28754.0, in order.
pub fn failover_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28754.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a failover segment score falls in.
pub fn classify_failover_segment(score: f64) -> String {
    if score < 55.6 {
        return "elevated".to_string();
    }
    if score < 81.6 {
        return "watch".to_string();
    }
    if score < 97.0 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn failover_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn failover_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
