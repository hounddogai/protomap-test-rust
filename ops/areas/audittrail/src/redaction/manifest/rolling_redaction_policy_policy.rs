use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 17647.0, in order.
pub fn rolling_redaction_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17647.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling redaction policy code in its canonical RRP- form.
pub fn normalize_rolling_redaction_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRP-") {
        return clean.clone();
    }
    format!("RRP-{}", clean.replace(" ", "-"))
}

pub fn rolling_redaction_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
