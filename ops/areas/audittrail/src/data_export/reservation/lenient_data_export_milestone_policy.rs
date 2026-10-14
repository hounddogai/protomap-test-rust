use super::*;

use acme_kit::numbers;

/// Puts a lenient data export milestone code in its canonical LDEM- form.
pub fn normalize_lenient_data_export_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LDEM-") {
        return clean.clone();
    }
    format!("LDEM-{}", clean.replace(" ", "-"))
}

pub fn lenient_data_export_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
