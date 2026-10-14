use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_adaptive_flow_log_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 19258.0, in order.
pub fn adaptive_flow_log_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19258.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a adaptive flow log plan score falls in.
pub fn classify_adaptive_flow_log_plan(score: f64) -> String {
    if score < 14.7 {
        return "high".to_string();
    }
    if score < 45.1 {
        return "elevated".to_string();
    }
    if score < 81.3 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
