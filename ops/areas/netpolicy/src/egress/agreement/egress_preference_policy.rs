use super::*;

use acme_kit::numbers;

pub fn egress_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a egress preference score falls in.
pub fn classify_egress_preference(score: f64) -> String {
    if score < 1.6 {
        return "critical".to_string();
    }
    if score < 2.5 {
        return "watch".to_string();
    }
    if score < 27.6 {
        return "high".to_string();
    }
    if score < 59.6 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Puts a egress preference code in its canonical EP- form.
pub fn normalize_egress_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EP-") {
        return clean.clone();
    }
    format!("EP-{}", clean.replace(" ", "-"))
}
