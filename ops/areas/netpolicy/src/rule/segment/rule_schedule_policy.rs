use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 78001.0, in order.
pub fn rule_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78001.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rule schedule score falls in.
pub fn classify_rule_schedule(score: f64) -> String {
    if score < 7.3 {
        return "critical".to_string();
    }
    if score < 53.1 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_rule_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
