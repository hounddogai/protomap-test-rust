use super::*;

/// Names the band a token review score falls in.
pub fn classify_token_review(score: f64) -> String {
    if score < 2.0 {
        return "minimal".to_string();
    }
    if score < 40.7 {
        return "watch".to_string();
    }
    if score < 49.1 {
        return "high".to_string();
    }
    if score < 80.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn token_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 50729.0, in order.
pub fn token_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50729.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a token review code in its canonical TR- form.
pub fn normalize_token_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}
