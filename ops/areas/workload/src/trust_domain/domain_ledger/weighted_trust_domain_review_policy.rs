use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn weighted_trust_domain_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 94425.0, in order.
pub fn weighted_trust_domain_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94425.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted trust domain review score falls in.
pub fn classify_weighted_trust_domain_review(score: f64) -> String {
    if score < 13.9 {
        return "medium".to_string();
    }
    if score < 29.7 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn merge_weighted_trust_domain_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
