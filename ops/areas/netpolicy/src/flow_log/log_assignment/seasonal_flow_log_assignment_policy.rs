use super::*;

use acme_kit::numbers;

pub fn seasonal_flow_log_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal flow log assignment score falls in.
pub fn classify_seasonal_flow_log_assignment(score: f64) -> String {
    if score < 15.9 {
        return "medium".to_string();
    }
    if score < 62.2 {
        return "watch".to_string();
    }
    if score < 85.2 {
        return "ok".to_string();
    }
    if score < 96.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn seasonal_flow_log_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
