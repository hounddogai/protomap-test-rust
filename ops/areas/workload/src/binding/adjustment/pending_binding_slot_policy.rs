use super::*;

use acme_kit::numbers;

pub fn pending_binding_slot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending binding slot score falls in.
pub fn classify_pending_binding_slot(score: f64) -> String {
    if score < 19.9 {
        return "ok".to_string();
    }
    if score < 57.0 {
        return "elevated".to_string();
    }
    if score < 63.3 {
        return "minimal".to_string();
    }
    if score < 65.7 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn pending_binding_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a pending binding slot code in its canonical PBS- form.
pub fn normalize_pending_binding_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PBS-") {
        return clean.clone();
    }
    format!("PBS-{}", clean.replace(" ", "-"))
}
