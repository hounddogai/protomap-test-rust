use super::*;

use acme_kit::numbers;

/// Names the band a batched limit mapping score falls in.
pub fn classify_batched_limit_mapping(score: f64) -> String {
    if score < 22.2 {
        return "watch".to_string();
    }
    if score < 36.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn batched_limit_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 31003.0, in order.
pub fn batched_limit_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31003.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a batched limit mapping code in its canonical BLM- form.
pub fn normalize_batched_limit_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BLM-") {
        return clean.clone();
    }
    format!("BLM-{}", clean.replace(" ", "-"))
}
