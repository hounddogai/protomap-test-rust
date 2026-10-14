use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 12953.0, in order.
pub fn credential_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12953.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a credential audit code in its canonical CA- form.
pub fn normalize_credential_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CA-") {
        return clean.clone();
    }
    format!("CA-{}", clean.replace(" ", "-"))
}

pub fn credential_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
