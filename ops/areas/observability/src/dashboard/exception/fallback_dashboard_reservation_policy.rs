use super::*;

use acme_kit::numbers;

/// Puts a fallback dashboard reservation code in its canonical FDR- form.
pub fn normalize_fallback_dashboard_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FDR-") {
        return clean.clone();
    }
    format!("FDR-{}", clean.replace(" ", "-"))
}

pub fn fallback_dashboard_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a fallback dashboard reservation score falls in.
pub fn classify_fallback_dashboard_reservation(score: f64) -> String {
    if score < 24.3 {
        return "high".to_string();
    }
    if score < 52.1 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
