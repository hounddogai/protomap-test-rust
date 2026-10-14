use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_dashboard_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 69409.0, in order.
pub fn dashboard_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69409.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a dashboard metric score falls in.
pub fn classify_dashboard_metric(score: f64) -> String {
    if score < 2.3 {
        return "ok".to_string();
    }
    if score < 27.7 {
        return "minimal".to_string();
    }
    if score < 39.3 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn dashboard_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
