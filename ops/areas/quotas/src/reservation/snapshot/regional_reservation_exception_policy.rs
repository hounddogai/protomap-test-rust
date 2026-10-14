use super::*;

use acme_kit::numbers;

pub fn regional_reservation_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a regional reservation exception score falls in.
pub fn classify_regional_reservation_exception(score: f64) -> String {
    if score < 68.0 {
        return "watch".to_string();
    }
    if score < 91.3 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a regional reservation exception code in its canonical RRE- form.
pub fn normalize_regional_reservation_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRE-") {
        return clean.clone();
    }
    format!("RRE-{}", clean.replace(" ", "-"))
}
