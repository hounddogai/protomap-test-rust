use super::*;

use acme_kit::numbers;

pub fn release_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn release_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a release rule score falls in.
pub fn classify_release_rule(score: f64) -> String {
    if score < 22.2 {
        return "minimal".to_string();
    }
    if score < 32.3 {
        return "low".to_string();
    }
    "watch".to_string()
}
