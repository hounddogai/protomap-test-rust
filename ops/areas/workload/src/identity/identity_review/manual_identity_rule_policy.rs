use super::*;

use acme_kit::numbers;

pub fn manual_identity_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Puts a manual identity rule code in its canonical MIR- form.
pub fn normalize_manual_identity_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MIR-") {
        return clean.clone();
    }
    format!("MIR-{}", clean.replace(" ", "-"))
}

pub fn manual_identity_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
