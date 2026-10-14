use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn reservation_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn merge_reservation_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a reservation variance score falls in.
pub fn classify_reservation_variance(score: f64) -> String {
    if score < 69.3 {
        return "medium".to_string();
    }
    if score < 74.0 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn reservation_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
