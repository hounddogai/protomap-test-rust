use super::*;

use acme_kit::numbers;

/// Names the band a fallback action item assignment score falls in.
pub fn classify_fallback_action_item_assignment(score: f64) -> String {
    if score < 11.1 {
        return "medium".to_string();
    }
    if score < 67.9 {
        return "ok".to_string();
    }
    if score < 70.2 {
        return "critical".to_string();
    }
    if score < 82.5 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn fallback_action_item_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
