use super::*;

use acme_kit::numbers;

/// Names the band a scheduled action item rollup score falls in.
pub fn classify_scheduled_action_item_rollup(score: f64) -> String {
    if score < 35.0 {
        return "watch".to_string();
    }
    if score < 82.3 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 94031.0, in order.
pub fn scheduled_action_item_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94031.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_action_item_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn scheduled_action_item_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
