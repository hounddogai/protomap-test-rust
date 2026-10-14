use super::*;

use acme_kit::numbers;

pub fn capped_verifier_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_verifier_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a capped verifier rollup code in its canonical CVR- form.
pub fn normalize_capped_verifier_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CVR-") {
        return clean.clone();
    }
    format!("CVR-{}", clean.replace(" ", "-"))
}
