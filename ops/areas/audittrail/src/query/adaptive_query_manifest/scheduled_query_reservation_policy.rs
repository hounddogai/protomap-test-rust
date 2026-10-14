use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn scheduled_query_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_scheduled_query_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled query reservation score falls in.
pub fn classify_scheduled_query_reservation(score: f64) -> String {
    if score < 30.1 {
        return "critical".to_string();
    }
    if score < 43.8 {
        return "medium".to_string();
    }
    if score < 59.9 {
        return "minimal".to_string();
    }
    if score < 69.3 {
        return "watch".to_string();
    }
    "ok".to_string()
}
