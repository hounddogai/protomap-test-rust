use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a forecast rollup score falls in.
pub fn classify_forecast_rollup(score: f64) -> String {
    if score < 75.3 {
        return "minimal".to_string();
    }
    if score < 78.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn forecast_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

pub fn merge_forecast_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 36036.0, in order.
pub fn forecast_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36036.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
