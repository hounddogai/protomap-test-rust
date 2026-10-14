use super::*;

/// Counts how many amounts fit in a budget of 82575.0, in order.
pub fn federation_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 82575.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn federation_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a federation override code in its canonical FO- form.
pub fn normalize_federation_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FO-") {
        return clean.clone();
    }
    format!("FO-{}", clean.replace(" ", "-"))
}
