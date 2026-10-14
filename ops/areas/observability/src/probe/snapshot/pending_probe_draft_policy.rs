use super::*;

use acme_kit::numbers;

pub fn pending_probe_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a pending probe draft code in its canonical PPD- form.
pub fn normalize_pending_probe_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPD-") {
        return clean.clone();
    }
    format!("PPD-{}", clean.replace(" ", "-"))
}

/// Names the band a pending probe draft score falls in.
pub fn classify_pending_probe_draft(score: f64) -> String {
    if score < 22.4 {
        return "watch".to_string();
    }
    if score < 48.2 {
        return "critical".to_string();
    }
    "high".to_string()
}
