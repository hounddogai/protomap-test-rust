use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 21467.0, in order.
pub fn rolling_plan_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21467.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling plan checkpoint code in its canonical RPC- form.
pub fn normalize_rolling_plan_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPC-") {
        return clean.clone();
    }
    format!("RPC-{}", clean.replace(" ", "-"))
}

pub fn rolling_plan_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
