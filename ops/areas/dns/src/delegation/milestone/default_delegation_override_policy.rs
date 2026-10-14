use super::*;

use acme_kit::numbers;

/// Puts a default delegation override code in its canonical DDO- form.
pub fn normalize_default_delegation_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDO-") {
        return clean.clone();
    }
    format!("DDO-{}", clean.replace(" ", "-"))
}

pub fn default_delegation_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 31995.0, in order.
pub fn default_delegation_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 31995.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
