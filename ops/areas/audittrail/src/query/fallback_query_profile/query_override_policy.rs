use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a query override score falls in.
pub fn classify_query_override(score: f64) -> String {
    if score < 60.3 {
        return "minimal".to_string();
    }
    if score < 66.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn merge_query_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 64364.0, in order.
pub fn query_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64364.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn query_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
