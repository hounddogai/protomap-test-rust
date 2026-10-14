use super::*;

/// Puts a allowlist estimate code in its canonical AE- form.
pub fn normalize_allowlist_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AE-") {
        return clean.clone();
    }
    format!("AE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 55679.0, in order.
pub fn allowlist_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55679.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn allowlist_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
