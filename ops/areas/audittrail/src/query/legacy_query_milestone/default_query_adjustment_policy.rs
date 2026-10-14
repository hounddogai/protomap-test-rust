use super::*;

use acme_kit::numbers;

/// Puts a default query adjustment code in its canonical DQA- form.
pub fn normalize_default_query_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQA-") {
        return clean.clone();
    }
    format!("DQA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 3295.0, in order.
pub fn default_query_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3295.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn default_query_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
