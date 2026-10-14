use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_usage_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn usage_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn usage_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a usage reservation score falls in.
pub fn classify_usage_reservation(score: f64) -> String {
    if score < 1.8 {
        return "low".to_string();
    }
    if score < 40.9 {
        return "watch".to_string();
    }
    if score < 46.7 {
        return "critical".to_string();
    }
    if score < 97.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
