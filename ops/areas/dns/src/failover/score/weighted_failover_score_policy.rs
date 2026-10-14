use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 90585.0, in order.
pub fn weighted_failover_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90585.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted failover score score falls in.
pub fn classify_weighted_failover_score(score: f64) -> String {
    if score < 9.6 {
        return "elevated".to_string();
    }
    if score < 28.0 {
        return "ok".to_string();
    }
    if score < 31.7 {
        return "low".to_string();
    }
    if score < 70.3 {
        return "critical".to_string();
    }
    "high".to_string()
}

pub fn merge_weighted_failover_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
