use super::*;

use acme_kit::numbers;

pub fn manual_forecast_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a manual forecast summary code in its canonical MFS- form.
pub fn normalize_manual_forecast_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MFS-") {
        return clean.clone();
    }
    format!("MFS-{}", clean.replace(" ", "-"))
}
