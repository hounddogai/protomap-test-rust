use super::*;

use acme_kit::numbers;

pub fn data_export_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.09;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn data_export_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a data export profile code in its canonical DEP- form.
pub fn normalize_data_export_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEP-") {
        return clean.clone();
    }
    format!("DEP-{}", clean.replace(" ", "-"))
}
