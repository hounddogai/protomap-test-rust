use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_trust_domain_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual trust domain score score falls in.
pub fn classify_manual_trust_domain_score(score: f64) -> String {
    if score < 27.0 {
        return "low".to_string();
    }
    if score < 56.3 {
        return "minimal".to_string();
    }
    if score < 76.0 {
        return "critical".to_string();
    }
    if score < 83.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 85971.0, in order.
pub fn manual_trust_domain_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85971.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
