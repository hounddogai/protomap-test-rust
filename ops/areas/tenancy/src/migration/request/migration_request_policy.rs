use super::*;

use acme_kit::numbers;

pub fn migration_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a migration request score falls in.
pub fn classify_migration_request(score: f64) -> String {
    if score < 18.4 {
        return "minimal".to_string();
    }
    if score < 22.6 {
        return "medium".to_string();
    }
    if score < 28.2 {
        return "elevated".to_string();
    }
    if score < 82.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn migration_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
