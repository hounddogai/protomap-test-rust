use super::*;

/// Puts a strict query agreement code in its canonical SQA- form.
pub fn normalize_strict_query_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQA-") {
        return clean.clone();
    }
    format!("SQA-{}", clean.replace(" ", "-"))
}

pub fn strict_query_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Counts how many amounts fit in a budget of 54071.0, in order.
pub fn strict_query_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54071.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
