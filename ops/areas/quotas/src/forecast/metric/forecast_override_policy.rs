use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_forecast_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 37417.0, in order.
pub fn forecast_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37417.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a forecast override score falls in.
pub fn classify_forecast_override(score: f64) -> String {
    if score < 63.0 {
        return "elevated".to_string();
    }
    if score < 77.9 {
        return "low".to_string();
    }
    if score < 85.8 {
        return "watch".to_string();
    }
    "ok".to_string()
}
