use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a scheduled dashboard summary score falls in.
pub fn classify_scheduled_dashboard_summary(score: f64) -> String {
    if score < 8.6 {
        return "high".to_string();
    }
    if score < 16.8 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 10169.0, in order.
pub fn scheduled_dashboard_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10169.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_dashboard_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_scheduled_dashboard_summary_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
