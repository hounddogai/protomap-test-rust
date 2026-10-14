use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tenant_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_tenant_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tenant quota score falls in.
pub fn classify_tenant_quota(score: f64) -> String {
    if score < 10.0 {
        return "elevated".to_string();
    }
    if score < 63.0 {
        return "high".to_string();
    }
    if score < 64.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 41283.0, in order.
pub fn tenant_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 41283.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
