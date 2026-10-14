use super::*;

use acme_kit::numbers;

pub fn legacy_action_item_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 33287.0, in order.
pub fn legacy_action_item_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33287.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_action_item_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.88;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy action item notice score falls in.
pub fn classify_legacy_action_item_notice(score: f64) -> String {
    if score < 3.4 {
        return "high".to_string();
    }
    if score < 10.0 {
        return "medium".to_string();
    }
    "low".to_string()
}
