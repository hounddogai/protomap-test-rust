use super::*;

use acme_kit::numbers;

/// Puts a default rollout bundle code in its canonical DRB- form.
pub fn normalize_default_rollout_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRB-") {
        return clean.clone();
    }
    format!("DRB-{}", clean.replace(" ", "-"))
}

pub fn default_rollout_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
