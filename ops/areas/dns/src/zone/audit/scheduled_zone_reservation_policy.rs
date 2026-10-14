use super::*;

use acme_kit::numbers;

pub fn scheduled_zone_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn scheduled_zone_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a scheduled zone reservation score falls in.
pub fn classify_scheduled_zone_reservation(score: f64) -> String {
    if score < 10.9 {
        return "ok".to_string();
    }
    if score < 19.1 {
        return "elevated".to_string();
    }
    if score < 42.1 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Puts a scheduled zone reservation code in its canonical SZR- form.
pub fn normalize_scheduled_zone_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SZR-") {
        return clean.clone();
    }
    format!("SZR-{}", clean.replace(" ", "-"))
}
