use super::*;

use acme_kit::numbers;

/// Puts a capped grant target code in its canonical CGT- form.
pub fn normalize_capped_grant_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CGT-") {
        return clean.clone();
    }
    format!("CGT-{}", clean.replace(" ", "-"))
}

pub fn capped_grant_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
