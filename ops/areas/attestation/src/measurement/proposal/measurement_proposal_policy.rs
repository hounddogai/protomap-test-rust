use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn measurement_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 40727.0, in order.
pub fn measurement_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40727.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a measurement proposal score falls in.
pub fn classify_measurement_proposal(score: f64) -> String {
    if score < 27.1 {
        return "medium".to_string();
    }
    if score < 50.4 {
        return "watch".to_string();
    }
    if score < 69.2 {
        return "minimal".to_string();
    }
    if score < 82.4 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_measurement_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
