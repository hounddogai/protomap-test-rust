use super::*;

use acme_kit::numbers;

pub fn strict_evidence_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a strict evidence entry code in its canonical SEE- form.
pub fn normalize_strict_evidence_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEE-") {
        return clean.clone();
    }
    format!("SEE-{}", clean.replace(" ", "-"))
}

/// Names the band a strict evidence entry score falls in.
pub fn classify_strict_evidence_entry(score: f64) -> String {
    if score < 49.8 {
        return "high".to_string();
    }
    if score < 70.6 {
        return "watch".to_string();
    }
    if score < 85.5 {
        return "low".to_string();
    }
    if score < 96.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn strict_evidence_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
