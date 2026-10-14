use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 7685.0, in order.
pub fn audit_event_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7685.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a audit event baseline score falls in.
pub fn classify_audit_event_baseline(score: f64) -> String {
    if score < 33.2 {
        return "watch".to_string();
    }
    if score < 80.2 {
        return "minimal".to_string();
    }
    if score < 84.2 {
        return "ok".to_string();
    }
    "medium".to_string()
}

pub fn merge_audit_event_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a audit event baseline code in its canonical AEB- form.
pub fn normalize_audit_event_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEB-") {
        return clean.clone();
    }
    format!("AEB-{}", clean.replace(" ", "-"))
}
