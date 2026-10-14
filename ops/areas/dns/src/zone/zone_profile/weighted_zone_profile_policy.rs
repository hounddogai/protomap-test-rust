use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 45489.0, in order.
pub fn weighted_zone_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45489.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_weighted_zone_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn weighted_zone_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a weighted zone profile score falls in.
pub fn classify_weighted_zone_profile(score: f64) -> String {
    if score < 63.1 {
        return "minimal".to_string();
    }
    if score < 68.1 {
        return "elevated".to_string();
    }
    if score < 80.8 {
        return "low".to_string();
    }
    if score < 83.9 {
        return "medium".to_string();
    }
    "ok".to_string()
}
