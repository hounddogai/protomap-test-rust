use super::*;

use acme_kit::numbers;

pub fn reservation_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a reservation bundle score falls in.
pub fn classify_reservation_bundle(score: f64) -> String {
    if score < 25.1 {
        return "elevated".to_string();
    }
    if score < 35.9 {
        return "high".to_string();
    }
    if score < 65.2 {
        return "minimal".to_string();
    }
    if score < 79.3 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Puts a reservation bundle code in its canonical RB- form.
pub fn normalize_reservation_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}
