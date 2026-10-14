use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 87428.0, in order.
pub fn weighted_alert_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87428.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted alert reservation score falls in.
pub fn classify_weighted_alert_reservation(score: f64) -> String {
    if score < 19.9 {
        return "ok".to_string();
    }
    if score < 26.4 {
        return "minimal".to_string();
    }
    if score < 83.0 {
        return "watch".to_string();
    }
    if score < 93.6 {
        return "medium".to_string();
    }
    "critical".to_string()
}

pub fn merge_weighted_alert_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
