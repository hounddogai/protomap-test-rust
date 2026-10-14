use super::*;

use acme_kit::numbers;

/// Puts a binding proposal code in its canonical BP- form.
pub fn normalize_binding_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BP-") {
        return clean.clone();
    }
    format!("BP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 39191.0, in order.
pub fn binding_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39191.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn binding_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
