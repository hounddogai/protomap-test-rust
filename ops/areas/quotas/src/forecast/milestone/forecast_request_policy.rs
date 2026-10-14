use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a forecast request score falls in.
pub fn classify_forecast_request(score: f64) -> String {
    if score < 13.5 {
        return "low".to_string();
    }
    if score < 94.3 {
        return "minimal".to_string();
    }
    if score < 94.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn merge_forecast_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 89472.0, in order.
pub fn forecast_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89472.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn forecast_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
