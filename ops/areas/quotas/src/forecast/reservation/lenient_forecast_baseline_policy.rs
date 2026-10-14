use super::*;

use acme_kit::numbers;

/// Puts a lenient forecast baseline code in its canonical LFB- form.
pub fn normalize_lenient_forecast_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFB-") {
        return clean.clone();
    }
    format!("LFB-{}", clean.replace(" ", "-"))
}

pub fn lenient_forecast_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
