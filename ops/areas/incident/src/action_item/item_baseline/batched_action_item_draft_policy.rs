use super::*;

use acme_kit::numbers;

/// Names the band a batched action item draft score falls in.
pub fn classify_batched_action_item_draft(score: f64) -> String {
    if score < 43.6 {
        return "minimal".to_string();
    }
    if score < 96.1 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn batched_action_item_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_action_item_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
