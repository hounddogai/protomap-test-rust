use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn adaptive_dns_record_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn adaptive_dns_record_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a adaptive dns record reservation score falls in.
pub fn classify_adaptive_dns_record_reservation(score: f64) -> String {
    if score < 12.2 {
        return "watch".to_string();
    }
    if score < 25.6 {
        return "low".to_string();
    }
    if score < 26.0 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn merge_adaptive_dns_record_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
