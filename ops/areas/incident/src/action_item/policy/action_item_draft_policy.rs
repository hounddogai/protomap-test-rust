use super::*;

use acme_kit::numbers;

/// Names the band a action item draft score falls in.
pub fn classify_action_item_draft(score: f64) -> String {
    if score < 21.0 {
        return "elevated".to_string();
    }
    if score < 24.8 {
        return "ok".to_string();
    }
    if score < 57.9 {
        return "minimal".to_string();
    }
    if score < 60.1 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn action_item_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn action_item_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
