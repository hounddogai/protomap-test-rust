use super::*;

/// Puts a credential request code in its canonical CR- form.
pub fn normalize_credential_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CR-") {
        return clean.clone();
    }
    format!("CR-{}", clean.replace(" ", "-"))
}

pub fn credential_request_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 90739.0, in order.
pub fn credential_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90739.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a credential request score falls in.
pub fn classify_credential_request(score: f64) -> String {
    if score < 47.7 {
        return "watch".to_string();
    }
    if score < 51.2 {
        return "high".to_string();
    }
    if score < 65.4 {
        return "minimal".to_string();
    }
    if score < 80.6 {
        return "critical".to_string();
    }
    "medium".to_string()
}
