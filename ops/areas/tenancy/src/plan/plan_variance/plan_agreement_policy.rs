use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a plan agreement score falls in.
pub fn classify_plan_agreement(score: f64) -> String {
    if score < 25.2 {
        return "low".to_string();
    }
    if score < 40.2 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 66631.0, in order.
pub fn plan_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66631.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_plan_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
