use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn failover_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn failover_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn merge_failover_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a failover reservation score falls in.
pub fn classify_failover_reservation(score: f64) -> String {
    if score < 38.4 {
        return "minimal".to_string();
    }
    if score < 50.5 {
        return "critical".to_string();
    }
    if score < 67.9 {
        return "low".to_string();
    }
    if score < 73.3 {
        return "medium".to_string();
    }
    "ok".to_string()
}
