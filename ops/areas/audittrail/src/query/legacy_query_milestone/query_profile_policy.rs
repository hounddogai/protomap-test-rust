use super::*;

use acme_kit::numbers;

pub fn query_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a query profile code in its canonical QP- form.
pub fn normalize_query_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QP-") {
        return clean.clone();
    }
    format!("QP-{}", clean.replace(" ", "-"))
}

/// Names the band a query profile score falls in.
pub fn classify_query_profile(score: f64) -> String {
    if score < 31.6 {
        return "critical".to_string();
    }
    if score < 81.1 {
        return "low".to_string();
    }
    "ok".to_string()
}
