use super::*;

use acme_kit::numbers;

/// Puts a regional canary allocation code in its canonical RCA- form.
pub fn normalize_regional_canary_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCA-") {
        return clean.clone();
    }
    format!("RCA-{}", clean.replace(" ", "-"))
}

pub fn regional_canary_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
