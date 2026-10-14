use super::*;

use acme_kit::numbers;

pub fn pending_dashboard_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a pending dashboard batch code in its canonical PDB- form.
pub fn normalize_pending_dashboard_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PDB-") {
        return clean.clone();
    }
    format!("PDB-{}", clean.replace(" ", "-"))
}
