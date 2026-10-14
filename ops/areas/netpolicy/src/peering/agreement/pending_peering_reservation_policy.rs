use super::*;

use acme_kit::numbers;

/// Names the band a pending peering reservation score falls in.
pub fn classify_pending_peering_reservation(score: f64) -> String {
    if score < 6.1 {
        return "elevated".to_string();
    }
    if score < 38.1 {
        return "watch".to_string();
    }
    if score < 70.0 {
        return "high".to_string();
    }
    if score < 72.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Puts a pending peering reservation code in its canonical PPR- form.
pub fn normalize_pending_peering_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPR-") {
        return clean.clone();
    }
    format!("PPR-{}", clean.replace(" ", "-"))
}

pub fn pending_peering_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
