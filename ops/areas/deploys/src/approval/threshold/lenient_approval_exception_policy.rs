use super::*;

use acme_kit::numbers;

pub fn lenient_approval_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient approval exception score falls in.
pub fn classify_lenient_approval_exception(score: f64) -> String {
    if score < 17.7 {
        return "minimal".to_string();
    }
    if score < 53.9 {
        return "ok".to_string();
    }
    if score < 74.5 {
        return "low".to_string();
    }
    if score < 96.4 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn lenient_approval_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}
