use super::*;

use acme_kit::numbers;

/// Names the band a capped delegation draft score falls in.
pub fn classify_capped_delegation_draft(score: f64) -> String {
    if score < 1.9 {
        return "ok".to_string();
    }
    if score < 5.6 {
        return "medium".to_string();
    }
    if score < 8.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn capped_delegation_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_delegation_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
