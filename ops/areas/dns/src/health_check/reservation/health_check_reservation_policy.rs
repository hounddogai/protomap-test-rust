use super::*;

use acme_kit::numbers;

/// Puts a health check reservation code in its canonical HCR- form.
pub fn normalize_health_check_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCR-") {
        return clean.clone();
    }
    format!("HCR-{}", clean.replace(" ", "-"))
}

pub fn health_check_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
