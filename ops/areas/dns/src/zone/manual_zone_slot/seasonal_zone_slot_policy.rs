use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_seasonal_zone_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal zone slot score falls in.
pub fn classify_seasonal_zone_slot(score: f64) -> String {
    if score < 19.2 {
        return "elevated".to_string();
    }
    if score < 34.1 {
        return "high".to_string();
    }
    if score < 42.0 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 51859.0, in order.
pub fn seasonal_zone_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51859.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a seasonal zone slot code in its canonical SZS- form.
pub fn normalize_seasonal_zone_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SZS-") {
        return clean.clone();
    }
    format!("SZS-{}", clean.replace(" ", "-"))
}
