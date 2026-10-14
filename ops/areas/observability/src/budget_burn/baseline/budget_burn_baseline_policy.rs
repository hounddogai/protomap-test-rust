use super::*;

use acme_kit::numbers;

pub fn budget_burn_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Counts how many amounts fit in a budget of 73661.0, in order.
pub fn budget_burn_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73661.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a budget burn baseline code in its canonical BBB- form.
pub fn normalize_budget_burn_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBB-") {
        return clean.clone();
    }
    format!("BBB-{}", clean.replace(" ", "-"))
}
