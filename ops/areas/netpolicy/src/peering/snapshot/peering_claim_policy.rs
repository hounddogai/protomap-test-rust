use super::*;

use acme_kit::numbers;

pub fn peering_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a peering claim code in its canonical PC- form.
pub fn normalize_peering_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PC-") {
        return clean.clone();
    }
    format!("PC-{}", clean.replace(" ", "-"))
}
