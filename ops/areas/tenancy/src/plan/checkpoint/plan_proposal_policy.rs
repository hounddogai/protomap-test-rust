use super::*;

/// Counts how many amounts fit in a budget of 74905.0, in order.
pub fn plan_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74905.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a plan proposal score falls in.
pub fn classify_plan_proposal(score: f64) -> String {
    if score < 46.3 {
        return "elevated".to_string();
    }
    if score < 57.8 {
        return "watch".to_string();
    }
    if score < 96.9 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Puts a plan proposal code in its canonical PP- form.
pub fn normalize_plan_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}

pub fn plan_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
