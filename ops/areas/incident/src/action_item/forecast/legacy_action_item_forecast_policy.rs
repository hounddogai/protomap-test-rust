use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn legacy_action_item_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy action item forecast score falls in.
pub fn classify_legacy_action_item_forecast(score: f64) -> String {
    if score < 94.8 {
        return "minimal".to_string();
    }
    if score < 97.1 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn merge_legacy_action_item_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 17532.0, in order.
pub fn legacy_action_item_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17532.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
