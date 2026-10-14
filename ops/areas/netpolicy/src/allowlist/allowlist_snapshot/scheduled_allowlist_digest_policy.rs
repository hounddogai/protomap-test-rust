use super::*;

/// Puts a scheduled allowlist digest code in its canonical SAD- form.
pub fn normalize_scheduled_allowlist_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAD-") {
        return clean.clone();
    }
    format!("SAD-{}", clean.replace(" ", "-"))
}

pub fn scheduled_allowlist_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 43797.0, in order.
pub fn scheduled_allowlist_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43797.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
