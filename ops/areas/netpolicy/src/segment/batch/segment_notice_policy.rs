use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn segment_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_segment_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a segment notice score falls in.
pub fn classify_segment_notice(score: f64) -> String {
    if score < 10.9 {
        return "critical".to_string();
    }
    if score < 52.0 {
        return "ok".to_string();
    }
    if score < 62.1 {
        return "high".to_string();
    }
    if score < 94.7 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 79551.0, in order.
pub fn segment_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 79551.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
