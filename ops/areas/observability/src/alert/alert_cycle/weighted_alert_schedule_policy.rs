use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a weighted alert schedule code in its canonical WAS- form.
pub fn normalize_weighted_alert_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WAS-") {
        return clean.clone();
    }
    format!("WAS-{}", clean.replace(" ", "-"))
}

pub fn merge_weighted_alert_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 60055.0, in order.
pub fn weighted_alert_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60055.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn weighted_alert_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
