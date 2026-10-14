use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 17529.0, in order.
pub fn batched_postmortem_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17529.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched postmortem proposal score falls in.
pub fn classify_batched_postmortem_proposal(score: f64) -> String {
    if score < 1.5 {
        return "watch".to_string();
    }
    if score < 3.7 {
        return "ok".to_string();
    }
    if score < 73.7 {
        return "critical".to_string();
    }
    if score < 97.1 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Puts a batched postmortem proposal code in its canonical BPP- form.
pub fn normalize_batched_postmortem_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BPP-") {
        return clean.clone();
    }
    format!("BPP-{}", clean.replace(" ", "-"))
}

pub fn merge_batched_postmortem_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
