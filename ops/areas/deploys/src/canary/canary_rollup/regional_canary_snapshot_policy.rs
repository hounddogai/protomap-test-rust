use super::*;

use acme_kit::numbers;

/// Puts a regional canary snapshot code in its canonical RCS- form.
pub fn normalize_regional_canary_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RCS-") {
        return clean.clone();
    }
    format!("RCS-{}", clean.replace(" ", "-"))
}

pub fn regional_canary_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn regional_canary_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
