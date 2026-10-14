use super::*;

use acme_kit::numbers;

pub fn strict_retention_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a strict retention profile score falls in.
pub fn classify_strict_retention_profile(score: f64) -> String {
    if score < 4.4 {
        return "minimal".to_string();
    }
    if score < 6.9 {
        return "low".to_string();
    }
    if score < 68.6 {
        return "watch".to_string();
    }
    if score < 73.2 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn strict_retention_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
