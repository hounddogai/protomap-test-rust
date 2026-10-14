use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 58870.0, in order.
pub fn tenant_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58870.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tenant_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a tenant template code in its canonical TT- form.
pub fn normalize_tenant_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TT-") {
        return clean.clone();
    }
    format!("TT-{}", clean.replace(" ", "-"))
}
