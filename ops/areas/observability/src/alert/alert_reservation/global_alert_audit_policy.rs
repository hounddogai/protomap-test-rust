use super::*;

use acme_kit::numbers;

/// Names the band a global alert audit score falls in.
pub fn classify_global_alert_audit(score: f64) -> String {
    if score < 19.8 {
        return "ok".to_string();
    }
    if score < 25.5 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn global_alert_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn global_alert_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
