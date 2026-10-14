use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn flow_log_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_flow_log_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a flow log forecast score falls in.
pub fn classify_flow_log_forecast(score: f64) -> String {
    if score < 13.8 {
        return "medium".to_string();
    }
    if score < 77.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 76671.0, in order.
pub fn flow_log_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76671.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
