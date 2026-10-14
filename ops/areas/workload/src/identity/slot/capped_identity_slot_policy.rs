use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 19829.0, in order.
pub fn capped_identity_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19829.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_identity_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a capped identity slot code in its canonical CIS- form.
pub fn normalize_capped_identity_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CIS-") {
        return clean.clone();
    }
    format!("CIS-{}", clean.replace(" ", "-"))
}
