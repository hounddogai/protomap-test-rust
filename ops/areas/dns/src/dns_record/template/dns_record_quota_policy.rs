use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn dns_record_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_dns_record_quota_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a dns record quota score falls in.
pub fn classify_dns_record_quota(score: f64) -> String {
    if score < 52.3 {
        return "low".to_string();
    }
    if score < 81.0 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 20006.0, in order.
pub fn dns_record_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20006.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
