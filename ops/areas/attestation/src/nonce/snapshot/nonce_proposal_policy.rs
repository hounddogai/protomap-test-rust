use super::*;

pub fn nonce_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 89940.0, in order.
pub fn nonce_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89940.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a nonce proposal code in its canonical NP- form.
pub fn normalize_nonce_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NP-") {
        return clean.clone();
    }
    format!("NP-{}", clean.replace(" ", "-"))
}
