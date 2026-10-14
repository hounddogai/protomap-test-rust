use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 9445.0, in order.
pub fn tenant_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9445.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_tenant_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tenant review code in its canonical TR- form.
pub fn normalize_tenant_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}

/// Names the band a tenant review score falls in.
pub fn classify_tenant_review(score: f64) -> String {
    if score < 10.4 {
        return "high".to_string();
    }
    if score < 46.7 {
        return "ok".to_string();
    }
    if score < 63.4 {
        return "critical".to_string();
    }
    if score < 72.0 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
