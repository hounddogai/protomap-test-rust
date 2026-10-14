use super::*;

use acme_kit::numbers;

/// Names the band a postmortem request score falls in.
pub fn classify_postmortem_request(score: f64) -> String {
    if score < 10.1 {
        return "minimal".to_string();
    }
    if score < 21.3 {
        return "low".to_string();
    }
    if score < 64.0 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn postmortem_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn postmortem_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn postmortem_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
