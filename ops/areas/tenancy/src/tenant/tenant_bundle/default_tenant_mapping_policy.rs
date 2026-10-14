use super::*;

use acme_kit::numbers;

pub fn default_tenant_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.17;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a default tenant mapping code in its canonical DTM- form.
pub fn normalize_default_tenant_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DTM-") {
        return clean.clone();
    }
    format!("DTM-{}", clean.replace(" ", "-"))
}
