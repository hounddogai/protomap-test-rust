use super::*;

use acme_kit::numbers;

pub fn seasonal_action_item_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn seasonal_action_item_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn seasonal_action_item_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal action item review score falls in.
pub fn classify_seasonal_action_item_review(score: f64) -> String {
    if score < 38.5 {
        return "critical".to_string();
    }
    if score < 40.5 {
        return "ok".to_string();
    }
    if score < 66.7 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
