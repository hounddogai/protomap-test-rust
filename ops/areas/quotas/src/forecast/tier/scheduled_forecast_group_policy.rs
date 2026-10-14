use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn scheduled_forecast_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled forecast group score falls in.
pub fn classify_scheduled_forecast_group(score: f64) -> String {
    if score < 1.3 {
        return "watch".to_string();
    }
    if score < 9.7 {
        return "critical".to_string();
    }
    if score < 71.4 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn merge_scheduled_forecast_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 28712.0, in order.
pub fn scheduled_forecast_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28712.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
