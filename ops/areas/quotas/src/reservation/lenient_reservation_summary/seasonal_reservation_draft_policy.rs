use super::*;

use acme_kit::numbers;

pub fn seasonal_reservation_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn seasonal_reservation_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a seasonal reservation draft code in its canonical SRD- form.
pub fn normalize_seasonal_reservation_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRD-") {
        return clean.clone();
    }
    format!("SRD-{}", clean.replace(" ", "-"))
}
