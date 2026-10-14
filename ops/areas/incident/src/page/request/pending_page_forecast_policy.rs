use super::*;

use acme_kit::numbers;

/// Puts a pending page forecast code in its canonical PPF- form.
pub fn normalize_pending_page_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPF-") {
        return clean.clone();
    }
    format!("PPF-{}", clean.replace(" ", "-"))
}

pub fn pending_page_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
