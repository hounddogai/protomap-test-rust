use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_capped_verifier_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped verifier baseline score falls in.
pub fn classify_capped_verifier_baseline(score: f64) -> String {
    if score < 34.1 {
        return "high".to_string();
    }
    if score < 36.4 {
        return "watch".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 78682.0, in order.
pub fn capped_verifier_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78682.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_verifier_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
