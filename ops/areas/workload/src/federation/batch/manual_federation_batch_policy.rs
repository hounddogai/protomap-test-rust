use super::*;

use acme_kit::numbers;

/// Puts a manual federation batch code in its canonical MFB- form.
pub fn normalize_manual_federation_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MFB-") {
        return clean.clone();
    }
    format!("MFB-{}", clean.replace(" ", "-"))
}

pub fn manual_federation_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
