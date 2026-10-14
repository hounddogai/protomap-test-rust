use super::*;

use acme_kit::numbers;

/// Puts a lenient rule request code in its canonical LRR- form.
pub fn normalize_lenient_rule_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRR-") {
        return clean.clone();
    }
    format!("LRR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 24267.0, in order.
pub fn lenient_rule_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24267.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_rule_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
