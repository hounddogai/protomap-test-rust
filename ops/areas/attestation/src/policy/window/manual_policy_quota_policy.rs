use super::*;

use acme_kit::numbers;

/// Puts a manual policy quota code in its canonical MPQ- form.
pub fn normalize_manual_policy_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPQ-") {
        return clean.clone();
    }
    format!("MPQ-{}", clean.replace(" ", "-"))
}

pub fn manual_policy_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
