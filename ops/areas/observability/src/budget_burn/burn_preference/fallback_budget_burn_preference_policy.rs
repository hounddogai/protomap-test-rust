use super::*;

use acme_kit::numbers;

pub fn fallback_budget_burn_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_budget_burn_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a fallback budget burn preference code in its canonical FBBP- form.
pub fn normalize_fallback_budget_burn_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FBBP-") {
        return clean.clone();
    }
    format!("FBBP-{}", clean.replace(" ", "-"))
}
