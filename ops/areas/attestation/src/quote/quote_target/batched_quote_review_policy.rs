use super::*;

use acme_kit::numbers;

pub fn batched_quote_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a batched quote review code in its canonical BQR- form.
pub fn normalize_batched_quote_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BQR-") {
        return clean.clone();
    }
    format!("BQR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 50648.0, in order.
pub fn batched_quote_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50648.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched quote review score falls in.
pub fn classify_batched_quote_review(score: f64) -> String {
    if score < 23.9 {
        return "ok".to_string();
    }
    if score < 34.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}
