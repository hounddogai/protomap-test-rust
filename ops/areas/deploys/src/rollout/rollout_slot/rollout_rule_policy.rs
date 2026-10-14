use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a rollout rule score falls in.
pub fn classify_rollout_rule(score: f64) -> String {
    if score < 72.2 {
        return "low".to_string();
    }
    if score < 76.4 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn merge_rollout_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 20421.0, in order.
pub fn rollout_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20421.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rollout rule code in its canonical RR- form.
pub fn normalize_rollout_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}
