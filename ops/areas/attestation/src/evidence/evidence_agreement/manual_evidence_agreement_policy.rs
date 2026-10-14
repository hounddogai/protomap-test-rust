use super::*;

use acme_kit::numbers;

pub fn manual_evidence_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a manual evidence agreement code in its canonical MEA- form.
pub fn normalize_manual_evidence_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MEA-") {
        return clean.clone();
    }
    format!("MEA-{}", clean.replace(" ", "-"))
}
