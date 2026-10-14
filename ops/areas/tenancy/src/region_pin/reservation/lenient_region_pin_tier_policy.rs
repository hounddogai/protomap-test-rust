use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a lenient region pin tier code in its canonical LRPT- form.
pub fn normalize_lenient_region_pin_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRPT-") {
        return clean.clone();
    }
    format!("LRPT-{}", clean.replace(" ", "-"))
}

pub fn merge_lenient_region_pin_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 93263.0, in order.
pub fn lenient_region_pin_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93263.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
