use super::*;

use acme_kit::numbers;

pub fn seasonal_query_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal query milestone score falls in.
pub fn classify_seasonal_query_milestone(score: f64) -> String {
    if score < 2.6 {
        return "watch".to_string();
    }
    if score < 50.3 {
        return "high".to_string();
    }
    if score < 83.8 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn seasonal_query_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
