use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_capped_data_export_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a capped data export batch score falls in.
pub fn classify_capped_data_export_batch(score: f64) -> String {
    if score < 21.2 {
        return "minimal".to_string();
    }
    if score < 21.3 {
        return "critical".to_string();
    }
    if score < 31.6 {
        return "high".to_string();
    }
    if score < 97.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 69743.0, in order.
pub fn capped_data_export_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69743.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_data_export_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
