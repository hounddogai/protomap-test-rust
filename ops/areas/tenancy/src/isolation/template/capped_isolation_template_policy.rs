use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a capped isolation template score falls in.
pub fn classify_capped_isolation_template(score: f64) -> String {
    if score < 25.9 {
        return "ok".to_string();
    }
    if score < 31.1 {
        return "critical".to_string();
    }
    if score < 33.1 {
        return "watch".to_string();
    }
    if score < 92.4 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_capped_isolation_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 56894.0, in order.
pub fn capped_isolation_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56894.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a capped isolation template code in its canonical CIT- form.
pub fn normalize_capped_isolation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CIT-") {
        return clean.clone();
    }
    format!("CIT-{}", clean.replace(" ", "-"))
}
