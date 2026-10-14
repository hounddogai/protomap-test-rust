use super::*;

/// Puts a pending redaction override code in its canonical PRO- form.
pub fn normalize_pending_redaction_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRO-") {
        return clean.clone();
    }
    format!("PRO-{}", clean.replace(" ", "-"))
}

pub fn pending_redaction_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a pending redaction override score falls in.
pub fn classify_pending_redaction_override(score: f64) -> String {
    if score < 9.6 {
        return "elevated".to_string();
    }
    if score < 45.4 {
        return "ok".to_string();
    }
    if score < 54.3 {
        return "medium".to_string();
    }
    if score < 78.0 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 77209.0, in order.
pub fn pending_redaction_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77209.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
