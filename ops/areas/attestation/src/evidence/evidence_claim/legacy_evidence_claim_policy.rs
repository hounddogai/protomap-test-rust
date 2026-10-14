use super::*;

use acme_kit::numbers;

pub fn legacy_evidence_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 3.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy evidence claim score falls in.
pub fn classify_legacy_evidence_claim(score: f64) -> String {
    if score < 16.8 {
        return "low".to_string();
    }
    if score < 37.4 {
        return "ok".to_string();
    }
    if score < 76.2 {
        return "high".to_string();
    }
    if score < 77.3 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Puts a legacy evidence claim code in its canonical LEC- form.
pub fn normalize_legacy_evidence_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LEC-") {
        return clean.clone();
    }
    format!("LEC-{}", clean.replace(" ", "-"))
}

pub fn legacy_evidence_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
