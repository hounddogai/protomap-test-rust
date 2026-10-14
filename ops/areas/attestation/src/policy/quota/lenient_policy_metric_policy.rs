use super::*;

use acme_kit::numbers;

pub fn lenient_policy_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn lenient_policy_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a lenient policy metric score falls in.
pub fn classify_lenient_policy_metric(score: f64) -> String {
    if score < 33.0 {
        return "watch".to_string();
    }
    if score < 57.8 {
        return "high".to_string();
    }
    if score < 68.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn lenient_policy_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
