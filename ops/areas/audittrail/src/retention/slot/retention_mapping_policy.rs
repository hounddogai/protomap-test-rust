use super::*;

use acme_kit::numbers;

pub fn retention_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a retention mapping code in its canonical RM- form.
pub fn normalize_retention_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RM-") {
        return clean.clone();
    }
    format!("RM-{}", clean.replace(" ", "-"))
}
