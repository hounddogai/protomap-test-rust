use super::*;

use acme_kit::numbers;

pub fn manual_rollback_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn manual_rollback_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a manual rollback profile score falls in.
pub fn classify_manual_rollback_profile(score: f64) -> String {
    if score < 73.2 {
        return "high".to_string();
    }
    if score < 77.5 {
        return "low".to_string();
    }
    "watch".to_string()
}
