use super::*;

use acme_kit::numbers;

pub fn plan_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a plan digest score falls in.
pub fn classify_plan_digest(score: f64) -> String {
    if score < 6.4 {
        return "elevated".to_string();
    }
    if score < 89.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn plan_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
