use super::*;

pub fn nonce_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Puts a nonce schedule code in its canonical NS- form.
pub fn normalize_nonce_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NS-") {
        return clean.clone();
    }
    format!("NS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 2482.0, in order.
pub fn nonce_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2482.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
