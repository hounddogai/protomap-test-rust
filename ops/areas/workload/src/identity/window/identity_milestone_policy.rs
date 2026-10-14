use super::*;

/// Counts how many amounts fit in a budget of 63184.0, in order.
pub fn identity_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63184.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a identity milestone code in its canonical IM- form.
pub fn normalize_identity_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IM-") {
        return clean.clone();
    }
    format!("IM-{}", clean.replace(" ", "-"))
}

pub fn identity_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
