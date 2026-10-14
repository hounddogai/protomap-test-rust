use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 87755.0, in order.
pub fn primary_rollout_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87755.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_primary_rollout_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a primary rollout proposal code in its canonical PRP- form.
pub fn normalize_primary_rollout_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRP-") {
        return clean.clone();
    }
    format!("PRP-{}", clean.replace(" ", "-"))
}

/// Names the band a primary rollout proposal score falls in.
pub fn classify_primary_rollout_proposal(score: f64) -> String {
    if score < 20.2 {
        return "critical".to_string();
    }
    if score < 30.3 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
