use super::*;

use acme_kit::numbers;

pub fn seasonal_postmortem_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a seasonal postmortem rollup code in its canonical SPR- form.
pub fn normalize_seasonal_postmortem_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPR-") {
        return clean.clone();
    }
    format!("SPR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 86159.0, in order.
pub fn seasonal_postmortem_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86159.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
