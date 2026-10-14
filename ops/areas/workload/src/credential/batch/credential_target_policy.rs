use super::*;

/// Puts a credential target code in its canonical CT- form.
pub fn normalize_credential_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CT-") {
        return clean.clone();
    }
    format!("CT-{}", clean.replace(" ", "-"))
}

pub fn credential_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 49634.0, in order.
pub fn credential_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49634.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
