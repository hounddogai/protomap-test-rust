use super::*;

use acme_kit::numbers;

/// Names the band a rolling credential slot score falls in.
pub fn classify_rolling_credential_slot(score: f64) -> String {
    if score < 10.0 {
        return "low".to_string();
    }
    if score < 20.0 {
        return "critical".to_string();
    }
    if score < 44.0 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn rolling_credential_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn rolling_credential_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
