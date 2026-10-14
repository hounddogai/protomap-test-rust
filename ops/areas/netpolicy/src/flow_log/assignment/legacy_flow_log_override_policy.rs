use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a legacy flow log override code in its canonical LFLO- form.
pub fn normalize_legacy_flow_log_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFLO-") {
        return clean.clone();
    }
    format!("LFLO-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy flow log override score falls in.
pub fn classify_legacy_flow_log_override(score: f64) -> String {
    if score < 47.6 {
        return "high".to_string();
    }
    if score < 78.6 {
        return "ok".to_string();
    }
    if score < 93.1 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn merge_legacy_flow_log_override_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 92283.0, in order.
pub fn legacy_flow_log_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92283.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
