use super::*;

use acme_kit::numbers;

pub fn pending_alert_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn pending_alert_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn pending_alert_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a pending alert entry score falls in.
pub fn classify_pending_alert_entry(score: f64) -> String {
    if score < 1.1 {
        return "elevated".to_string();
    }
    if score < 41.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}
