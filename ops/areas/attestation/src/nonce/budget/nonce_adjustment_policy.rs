use super::*;

pub fn nonce_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a nonce adjustment code in its canonical NA- form.
pub fn normalize_nonce_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NA-") {
        return clean.clone();
    }
    format!("NA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 91645.0, in order.
pub fn nonce_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91645.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
