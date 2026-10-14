use super::*;

use acme_kit::numbers;

pub fn regional_evidence_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a regional evidence manifest code in its canonical REM- form.
pub fn normalize_regional_evidence_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("REM-") {
        return clean.clone();
    }
    format!("REM-{}", clean.replace(" ", "-"))
}
