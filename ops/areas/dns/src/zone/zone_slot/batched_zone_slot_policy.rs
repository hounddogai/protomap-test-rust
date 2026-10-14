use super::*;

use acme_kit::numbers;

pub fn batched_zone_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_zone_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 74218.0, in order.
pub fn batched_zone_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74218.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched zone slot score falls in.
pub fn classify_batched_zone_slot(score: f64) -> String {
    if score < 14.5 {
        return "critical".to_string();
    }
    if score < 21.7 {
        return "high".to_string();
    }
    if score < 47.0 {
        return "medium".to_string();
    }
    "watch".to_string()
}
