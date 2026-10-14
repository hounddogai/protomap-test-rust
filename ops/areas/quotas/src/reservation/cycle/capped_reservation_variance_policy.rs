use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 9345.0, in order.
pub fn capped_reservation_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9345.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_reservation_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a capped reservation variance score falls in.
pub fn classify_capped_reservation_variance(score: f64) -> String {
    if score < 19.3 {
        return "medium".to_string();
    }
    if score < 55.5 {
        return "elevated".to_string();
    }
    if score < 56.8 {
        return "critical".to_string();
    }
    if score < 97.4 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_capped_reservation_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
