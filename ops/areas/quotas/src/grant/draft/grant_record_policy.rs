use super::*;

use acme_kit::numbers;

pub fn grant_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn grant_record_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a grant record score falls in.
pub fn classify_grant_record(score: f64) -> String {
    if score < 53.5 {
        return "watch".to_string();
    }
    if score < 64.4 {
        return "critical".to_string();
    }
    if score < 69.1 {
        return "low".to_string();
    }
    if score < 70.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}
