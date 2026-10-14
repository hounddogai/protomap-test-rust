use super::*;

/// Names the band a capped query agreement score falls in.
pub fn classify_capped_query_agreement(score: f64) -> String {
    if score < 27.9 {
        return "low".to_string();
    }
    if score < 37.7 {
        return "ok".to_string();
    }
    if score < 78.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Puts a capped query agreement code in its canonical CQA- form.
pub fn normalize_capped_query_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CQA-") {
        return clean.clone();
    }
    format!("CQA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 51074.0, in order.
pub fn capped_query_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51074.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_query_agreement_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
