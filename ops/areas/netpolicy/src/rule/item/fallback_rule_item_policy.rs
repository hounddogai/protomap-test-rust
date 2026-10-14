use super::*;

use acme_kit::numbers;

/// Puts a fallback rule item code in its canonical FRI- form.
pub fn normalize_fallback_rule_item_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRI-") {
        return clean.clone();
    }
    format!("FRI-{}", clean.replace(" ", "-"))
}

pub fn fallback_rule_item_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 5894.0, in order.
pub fn fallback_rule_item_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5894.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
