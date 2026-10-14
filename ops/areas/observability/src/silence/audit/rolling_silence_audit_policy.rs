use super::*;

use acme_kit::numbers;

/// Names the band a rolling silence audit score falls in.
pub fn classify_rolling_silence_audit(score: f64) -> String {
    if score < 56.1 {
        return "medium".to_string();
    }
    if score < 81.5 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn rolling_silence_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn rolling_silence_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
