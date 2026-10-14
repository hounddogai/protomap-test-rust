use super::*;

use acme_kit::numbers;

/// Names the band a adaptive slo tier score falls in.
pub fn classify_adaptive_slo_tier(score: f64) -> String {
    if score < 16.8 {
        return "high".to_string();
    }
    if score < 58.5 {
        return "medium".to_string();
    }
    if score < 71.6 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn adaptive_slo_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn adaptive_slo_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Puts a adaptive slo tier code in its canonical AST- form.
pub fn normalize_adaptive_slo_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AST-") {
        return clean.clone();
    }
    format!("AST-{}", clean.replace(" ", "-"))
}
