use super::*;

use acme_kit::numbers;

/// Puts a redaction claim code in its canonical RC- form.
pub fn normalize_redaction_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}

/// Names the band a redaction claim score falls in.
pub fn classify_redaction_claim(score: f64) -> String {
    if score < 24.6 {
        return "ok".to_string();
    }
    if score < 57.4 {
        return "minimal".to_string();
    }
    if score < 91.7 {
        return "high".to_string();
    }
    if score < 92.0 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn redaction_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn redaction_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
