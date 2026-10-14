use super::*;

use acme_kit::numbers;

/// Puts a adaptive credential variance code in its canonical ACV- form.
pub fn normalize_adaptive_credential_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ACV-") {
        return clean.clone();
    }
    format!("ACV-{}", clean.replace(" ", "-"))
}

pub fn adaptive_credential_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 72607.0, in order.
pub fn adaptive_credential_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72607.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
