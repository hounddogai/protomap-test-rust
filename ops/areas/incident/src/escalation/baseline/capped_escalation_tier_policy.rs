use super::*;

use acme_kit::numbers;

pub fn capped_escalation_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a capped escalation tier score falls in.
pub fn classify_capped_escalation_tier(score: f64) -> String {
    if score < 45.0 {
        return "high".to_string();
    }
    if score < 50.1 {
        return "watch".to_string();
    }
    if score < 89.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Puts a capped escalation tier code in its canonical CET- form.
pub fn normalize_capped_escalation_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CET-") {
        return clean.clone();
    }
    format!("CET-{}", clean.replace(" ", "-"))
}

pub fn capped_escalation_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
