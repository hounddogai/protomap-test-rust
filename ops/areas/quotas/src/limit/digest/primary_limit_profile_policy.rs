use super::*;

use acme_kit::numbers;

/// Puts a primary limit profile code in its canonical PLP- form.
pub fn normalize_primary_limit_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PLP-") {
        return clean.clone();
    }
    format!("PLP-{}", clean.replace(" ", "-"))
}

pub fn primary_limit_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 59091.0, in order.
pub fn primary_limit_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59091.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
