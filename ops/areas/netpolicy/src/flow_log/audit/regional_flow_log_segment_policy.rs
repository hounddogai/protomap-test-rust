use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_regional_flow_log_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a regional flow log segment score falls in.
pub fn classify_regional_flow_log_segment(score: f64) -> String {
    if score < 18.9 {
        return "watch".to_string();
    }
    if score < 19.3 {
        return "medium".to_string();
    }
    if score < 61.8 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 84485.0, in order.
pub fn regional_flow_log_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84485.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a regional flow log segment code in its canonical RFLS- form.
pub fn normalize_regional_flow_log_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFLS-") {
        return clean.clone();
    }
    format!("RFLS-{}", clean.replace(" ", "-"))
}
