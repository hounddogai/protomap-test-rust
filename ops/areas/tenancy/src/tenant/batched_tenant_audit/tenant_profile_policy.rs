use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tenant_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tenant profile score falls in.
pub fn classify_tenant_profile(score: f64) -> String {
    if score < 13.4 {
        return "ok".to_string();
    }
    if score < 67.7 {
        return "high".to_string();
    }
    if score < 71.5 {
        return "medium".to_string();
    }
    if score < 87.4 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 67995.0, in order.
pub fn tenant_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67995.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
