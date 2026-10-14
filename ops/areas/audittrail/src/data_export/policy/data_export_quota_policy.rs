use super::*;

use acme_kit::numbers;

/// Puts a data export quota code in its canonical DEQ- form.
pub fn normalize_data_export_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEQ-") {
        return clean.clone();
    }
    format!("DEQ-{}", clean.replace(" ", "-"))
}

pub fn data_export_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
