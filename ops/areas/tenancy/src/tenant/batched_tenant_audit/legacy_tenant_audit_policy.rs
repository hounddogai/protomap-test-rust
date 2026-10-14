use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Counts how many amounts fit in a budget of 62220.0, in order.
pub fn legacy_tenant_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62220.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn legacy_tenant_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn merge_legacy_tenant_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a legacy tenant audit code in its canonical LTA- form.
pub fn normalize_legacy_tenant_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LTA-") {
        return clean.clone();
    }
    format!("LTA-{}", clean.replace(" ", "-"))
}
