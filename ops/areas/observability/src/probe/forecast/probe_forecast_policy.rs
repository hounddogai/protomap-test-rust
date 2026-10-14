use super::*;

use acme_kit::numbers;

pub fn probe_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a probe forecast code in its canonical PF- form.
pub fn normalize_probe_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PF-") {
        return clean.clone();
    }
    format!("PF-{}", clean.replace(" ", "-"))
}
