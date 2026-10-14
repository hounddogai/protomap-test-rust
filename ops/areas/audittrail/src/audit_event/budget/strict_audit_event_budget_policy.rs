use super::*;

use acme_kit::numbers;

pub fn strict_audit_event_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a strict audit event budget score falls in.
pub fn classify_strict_audit_event_budget(score: f64) -> String {
    if score < 24.2 {
        return "elevated".to_string();
    }
    if score < 34.3 {
        return "ok".to_string();
    }
    if score < 83.0 {
        return "watch".to_string();
    }
    if score < 84.6 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn strict_audit_event_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a strict audit event budget code in its canonical SAEB- form.
pub fn normalize_strict_audit_event_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SAEB-") {
        return clean.clone();
    }
    format!("SAEB-{}", clean.replace(" ", "-"))
}
