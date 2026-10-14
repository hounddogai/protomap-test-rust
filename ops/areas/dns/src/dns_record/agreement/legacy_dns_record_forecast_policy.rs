use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_legacy_dns_record_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 94572.0, in order.
pub fn legacy_dns_record_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94572.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a legacy dns record forecast code in its canonical LDRF- form.
pub fn normalize_legacy_dns_record_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDRF-") {
        return clean.clone();
    }
    format!("LDRF-{}", clean.replace(" ", "-"))
}
