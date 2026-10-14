use super::*;

use acme_kit::numbers;

pub fn capped_verifier_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a capped verifier assignment code in its canonical CVA- form.
pub fn normalize_capped_verifier_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CVA-") {
        return clean.clone();
    }
    format!("CVA-{}", clean.replace(" ", "-"))
}
