use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a default policy rule score falls in.
pub fn classify_default_policy_rule(score: f64) -> String {
    if score < 4.8 {
        return "low".to_string();
    }
    if score < 57.9 {
        return "elevated".to_string();
    }
    if score < 76.4 {
        return "ok".to_string();
    }
    if score < 86.7 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 61163.0, in order.
pub fn default_policy_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61163.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a default policy rule code in its canonical DPR- form.
pub fn normalize_default_policy_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DPR-") {
        return clean.clone();
    }
    format!("DPR-{}", clean.replace(" ", "-"))
}

pub fn merge_default_policy_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
