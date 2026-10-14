use super::*;

use acme_kit::numbers;

/// Puts a seasonal usage variance code in its canonical SUV- form.
pub fn normalize_seasonal_usage_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SUV-") {
        return clean.clone();
    }
    format!("SUV-{}", clean.replace(" ", "-"))
}

/// Names the band a seasonal usage variance score falls in.
pub fn classify_seasonal_usage_variance(score: f64) -> String {
    if score < 19.4 {
        return "medium".to_string();
    }
    if score < 25.9 {
        return "high".to_string();
    }
    if score < 52.0 {
        return "watch".to_string();
    }
    if score < 58.8 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn seasonal_usage_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.87;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn seasonal_usage_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
