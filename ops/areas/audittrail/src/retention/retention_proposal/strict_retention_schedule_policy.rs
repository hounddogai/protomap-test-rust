use super::*;

use acme_kit::numbers;

pub fn strict_retention_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Names the band a strict retention schedule score falls in.
pub fn classify_strict_retention_schedule(score: f64) -> String {
    if score < 14.0 {
        return "watch".to_string();
    }
    if score < 45.2 {
        return "elevated".to_string();
    }
    if score < 64.3 {
        return "high".to_string();
    }
    if score < 72.8 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn strict_retention_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
