use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_fallback_evidence_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a fallback evidence agreement code in its canonical FEA- form.
pub fn normalize_fallback_evidence_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FEA-") {
        return clean.clone();
    }
    format!("FEA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 1111.0, in order.
pub fn fallback_evidence_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1111.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
