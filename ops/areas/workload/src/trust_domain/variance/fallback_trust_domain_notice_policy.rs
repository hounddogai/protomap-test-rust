use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a fallback trust domain notice code in its canonical FTDN- form.
pub fn normalize_fallback_trust_domain_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FTDN-") {
        return clean.clone();
    }
    format!("FTDN-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 5426.0, in order.
pub fn fallback_trust_domain_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5426.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback trust domain notice score falls in.
pub fn classify_fallback_trust_domain_notice(score: f64) -> String {
    if score < 15.3 {
        return "minimal".to_string();
    }
    if score < 20.5 {
        return "ok".to_string();
    }
    if score < 24.3 {
        return "critical".to_string();
    }
    if score < 87.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn merge_fallback_trust_domain_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
