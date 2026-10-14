use super::*;

use acme_kit::numbers;

/// Puts a primary slo reservation code in its canonical PSR- form.
pub fn normalize_primary_slo_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSR-") {
        return clean.clone();
    }
    format!("PSR-{}", clean.replace(" ", "-"))
}

pub fn primary_slo_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a primary slo reservation score falls in.
pub fn classify_primary_slo_reservation(score: f64) -> String {
    if score < 31.2 {
        return "watch".to_string();
    }
    if score < 37.9 {
        return "medium".to_string();
    }
    if score < 65.3 {
        return "ok".to_string();
    }
    "high".to_string()
}
