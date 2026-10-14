use super::*;

use acme_kit::numbers;

/// Puts a pending approval checkpoint code in its canonical PAC- form.
pub fn normalize_pending_approval_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PAC-") {
        return clean.clone();
    }
    format!("PAC-{}", clean.replace(" ", "-"))
}

/// Names the band a pending approval checkpoint score falls in.
pub fn classify_pending_approval_checkpoint(score: f64) -> String {
    if score < 66.2 {
        return "critical".to_string();
    }
    if score < 68.3 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn pending_approval_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
