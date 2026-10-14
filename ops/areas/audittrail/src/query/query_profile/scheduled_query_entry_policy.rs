use super::*;

use acme_kit::numbers;

/// Names the band a scheduled query entry score falls in.
pub fn classify_scheduled_query_entry(score: f64) -> String {
    if score < 53.6 {
        return "minimal".to_string();
    }
    if score < 80.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn scheduled_query_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn scheduled_query_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
