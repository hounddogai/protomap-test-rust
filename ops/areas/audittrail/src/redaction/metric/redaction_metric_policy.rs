use super::*;

/// Counts how many amounts fit in a budget of 1415.0, in order.
pub fn redaction_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1415.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a redaction metric score falls in.
pub fn classify_redaction_metric(score: f64) -> String {
    if score < 7.3 {
        return "elevated".to_string();
    }
    if score < 16.1 {
        return "ok".to_string();
    }
    if score < 92.2 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Puts a redaction metric code in its canonical RM- form.
pub fn normalize_redaction_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RM-") {
        return clean.clone();
    }
    format!("RM-{}", clean.replace(" ", "-"))
}

pub fn redaction_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
