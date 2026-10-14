use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_measurement_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a measurement tier score falls in.
pub fn classify_measurement_tier(score: f64) -> String {
    if score < 14.1 {
        return "minimal".to_string();
    }
    if score < 35.7 {
        return "critical".to_string();
    }
    if score < 71.2 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 40010.0, in order.
pub fn measurement_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40010.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
