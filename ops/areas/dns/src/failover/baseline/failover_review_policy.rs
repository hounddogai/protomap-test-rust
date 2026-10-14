use super::*;

pub fn failover_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a failover review code in its canonical FR- form.
pub fn normalize_failover_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FR-") {
        return clean.clone();
    }
    format!("FR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 67908.0, in order.
pub fn failover_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67908.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
