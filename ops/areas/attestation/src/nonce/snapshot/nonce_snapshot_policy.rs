use super::*;

/// Puts a nonce snapshot code in its canonical NS- form.
pub fn normalize_nonce_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NS-") {
        return clean.clone();
    }
    format!("NS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 52055.0, in order.
pub fn nonce_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52055.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn nonce_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
