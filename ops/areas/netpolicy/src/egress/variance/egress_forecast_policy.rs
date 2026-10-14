use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a egress forecast code in its canonical EF- form.
pub fn normalize_egress_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EF-") {
        return clean.clone();
    }
    format!("EF-{}", clean.replace(" ", "-"))
}

pub fn merge_egress_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn egress_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 3007.0, in order.
pub fn egress_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3007.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
