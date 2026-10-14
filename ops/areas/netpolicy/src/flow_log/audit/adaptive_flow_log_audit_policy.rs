use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_adaptive_flow_log_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn adaptive_flow_log_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a adaptive flow log audit code in its canonical AFLA- form.
pub fn normalize_adaptive_flow_log_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFLA-") {
        return clean.clone();
    }
    format!("AFLA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 66423.0, in order.
pub fn adaptive_flow_log_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66423.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
