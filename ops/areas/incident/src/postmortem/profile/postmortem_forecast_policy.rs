use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 10292.0, in order.
pub fn postmortem_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10292.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a postmortem forecast code in its canonical PF- form.
pub fn normalize_postmortem_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PF-") {
        return clean.clone();
    }
    format!("PF-{}", clean.replace(" ", "-"))
}

pub fn merge_postmortem_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a postmortem forecast score falls in.
pub fn classify_postmortem_forecast(score: f64) -> String {
    if score < 42.0 {
        return "critical".to_string();
    }
    if score < 52.8 {
        return "ok".to_string();
    }
    if score < 57.8 {
        return "minimal".to_string();
    }
    if score < 76.9 {
        return "watch".to_string();
    }
    "high".to_string()
}
