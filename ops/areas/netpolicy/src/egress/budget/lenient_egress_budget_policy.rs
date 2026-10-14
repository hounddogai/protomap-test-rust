use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 7159.0, in order.
pub fn lenient_egress_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7159.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient egress budget score falls in.
pub fn classify_lenient_egress_budget(score: f64) -> String {
    if score < 35.7 {
        return "medium".to_string();
    }
    if score < 51.9 {
        return "ok".to_string();
    }
    if score < 57.5 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn merge_lenient_egress_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
