use super::*;

use acme_kit::numbers;

pub fn lenient_measurement_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a lenient measurement notice code in its canonical LMN- form.
pub fn normalize_lenient_measurement_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LMN-") {
        return clean.clone();
    }
    format!("LMN-{}", clean.replace(" ", "-"))
}
