use super::*;

use acme_kit::numbers;

pub fn isolation_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn isolation_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a isolation mapping score falls in.
pub fn classify_isolation_mapping(score: f64) -> String {
    if score < 7.9 {
        return "minimal".to_string();
    }
    if score < 32.2 {
        return "medium".to_string();
    }
    if score < 35.5 {
        return "watch".to_string();
    }
    if score < 57.0 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
