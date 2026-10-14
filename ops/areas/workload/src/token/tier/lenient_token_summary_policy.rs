use super::*;

use acme_kit::numbers;

pub fn lenient_token_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn lenient_token_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.78;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient token summary score falls in.
pub fn classify_lenient_token_summary(score: f64) -> String {
    if score < 72.4 {
        return "ok".to_string();
    }
    if score < 93.1 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn lenient_token_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
