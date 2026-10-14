use super::*;

use acme_kit::numbers;

/// Puts a adaptive evidence estimate code in its canonical AEE- form.
pub fn normalize_adaptive_evidence_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AEE-") {
        return clean.clone();
    }
    format!("AEE-{}", clean.replace(" ", "-"))
}

/// Names the band a adaptive evidence estimate score falls in.
pub fn classify_adaptive_evidence_estimate(score: f64) -> String {
    if score < 18.6 {
        return "low".to_string();
    }
    if score < 37.0 {
        return "medium".to_string();
    }
    if score < 86.5 {
        return "ok".to_string();
    }
    if score < 96.1 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn adaptive_evidence_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn adaptive_evidence_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
