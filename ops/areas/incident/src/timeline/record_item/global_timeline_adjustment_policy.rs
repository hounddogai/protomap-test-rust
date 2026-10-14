use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a global timeline adjustment code in its canonical GTA- form.
pub fn normalize_global_timeline_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GTA-") {
        return clean.clone();
    }
    format!("GTA-{}", clean.replace(" ", "-"))
}

pub fn merge_global_timeline_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 84140.0, in order.
pub fn global_timeline_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84140.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a global timeline adjustment score falls in.
pub fn classify_global_timeline_adjustment(score: f64) -> String {
    if score < 20.4 {
        return "minimal".to_string();
    }
    if score < 72.6 {
        return "low".to_string();
    }
    if score < 78.0 {
        return "critical".to_string();
    }
    "ok".to_string()
}
