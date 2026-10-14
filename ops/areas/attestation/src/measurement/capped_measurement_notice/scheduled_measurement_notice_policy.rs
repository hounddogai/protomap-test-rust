use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a scheduled measurement notice code in its canonical SMN- form.
pub fn normalize_scheduled_measurement_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SMN-") {
        return clean.clone();
    }
    format!("SMN-{}", clean.replace(" ", "-"))
}

/// Names the band a scheduled measurement notice score falls in.
pub fn classify_scheduled_measurement_notice(score: f64) -> String {
    if score < 62.2 {
        return "critical".to_string();
    }
    if score < 75.4 {
        return "watch".to_string();
    }
    if score < 79.5 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 6290.0, in order.
pub fn scheduled_measurement_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6290.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_scheduled_measurement_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
