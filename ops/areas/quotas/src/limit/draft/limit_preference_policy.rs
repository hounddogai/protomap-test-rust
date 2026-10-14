use super::*;

use acme_kit::numbers;

pub fn limit_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 86026.0, in order.
pub fn limit_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86026.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a limit preference code in its canonical LP- form.
pub fn normalize_limit_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LP-") {
        return clean.clone();
    }
    format!("LP-{}", clean.replace(" ", "-"))
}
