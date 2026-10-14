use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_isolation_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a isolation audit score falls in.
pub fn classify_isolation_audit(score: f64) -> String {
    if score < 31.2 {
        return "medium".to_string();
    }
    if score < 34.0 {
        return "low".to_string();
    }
    if score < 50.9 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 85039.0, in order.
pub fn isolation_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85039.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a isolation audit code in its canonical IA- form.
pub fn normalize_isolation_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IA-") {
        return clean.clone();
    }
    format!("IA-{}", clean.replace(" ", "-"))
}
