use super::*;

use acme_kit::numbers;

/// Puts a forecast quota code in its canonical FQ- form.
pub fn normalize_forecast_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FQ-") {
        return clean.clone();
    }
    format!("FQ-{}", clean.replace(" ", "-"))
}

pub fn forecast_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a forecast quota score falls in.
pub fn classify_forecast_quota(score: f64) -> String {
    if score < 22.2 {
        return "watch".to_string();
    }
    if score < 44.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn forecast_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
