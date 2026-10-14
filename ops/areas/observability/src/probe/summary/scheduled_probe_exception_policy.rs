use super::*;

use acme_kit::numbers;

pub fn scheduled_probe_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.4;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn scheduled_probe_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a scheduled probe exception code in its canonical SPE- form.
pub fn normalize_scheduled_probe_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPE-") {
        return clean.clone();
    }
    format!("SPE-{}", clean.replace(" ", "-"))
}
