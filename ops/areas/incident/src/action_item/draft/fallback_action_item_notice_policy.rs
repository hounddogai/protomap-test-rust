use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a fallback action item notice code in its canonical FAIN- form.
pub fn normalize_fallback_action_item_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAIN-") {
        return clean.clone();
    }
    format!("FAIN-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 85712.0, in order.
pub fn fallback_action_item_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85712.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_fallback_action_item_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
