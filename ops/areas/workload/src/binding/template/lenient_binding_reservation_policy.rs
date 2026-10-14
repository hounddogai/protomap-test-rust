use super::*;

use acme_kit::numbers;

/// Names the band a lenient binding reservation score falls in.
pub fn classify_lenient_binding_reservation(score: f64) -> String {
    if score < 40.7 {
        return "watch".to_string();
    }
    if score < 51.7 {
        return "ok".to_string();
    }
    if score < 55.3 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a lenient binding reservation code in its canonical LBR- form.
pub fn normalize_lenient_binding_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LBR-") {
        return clean.clone();
    }
    format!("LBR-{}", clean.replace(" ", "-"))
}

pub fn lenient_binding_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
