use super::*;

use acme_kit::numbers;

pub fn manual_allowlist_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 51723.0, in order.
pub fn manual_allowlist_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51723.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a manual allowlist exception code in its canonical MAE- form.
pub fn normalize_manual_allowlist_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MAE-") {
        return clean.clone();
    }
    format!("MAE-{}", clean.replace(" ", "-"))
}
