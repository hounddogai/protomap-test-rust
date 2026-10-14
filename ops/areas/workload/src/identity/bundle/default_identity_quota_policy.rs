use super::*;

use acme_kit::numbers;

/// Puts a default identity quota code in its canonical DIQ- form.
pub fn normalize_default_identity_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DIQ-") {
        return clean.clone();
    }
    format!("DIQ-{}", clean.replace(" ", "-"))
}

pub fn default_identity_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.4;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
