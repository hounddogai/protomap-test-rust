use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 2367.0, in order.
pub fn legacy_runbook_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2367.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_legacy_runbook_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy runbook agreement score falls in.
pub fn classify_legacy_runbook_agreement(score: f64) -> String {
    if score < 41.6 {
        return "watch".to_string();
    }
    if score < 87.2 {
        return "minimal".to_string();
    }
    if score < 96.2 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a legacy runbook agreement code in its canonical LRA- form.
pub fn normalize_legacy_runbook_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRA-") {
        return clean.clone();
    }
    format!("LRA-{}", clean.replace(" ", "-"))
}
