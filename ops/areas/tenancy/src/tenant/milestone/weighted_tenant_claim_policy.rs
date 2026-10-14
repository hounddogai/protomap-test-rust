use super::*;

use acme_kit::numbers;

pub fn weighted_tenant_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted tenant claim code in its canonical WTC- form.
pub fn normalize_weighted_tenant_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WTC-") {
        return clean.clone();
    }
    format!("WTC-{}", clean.replace(" ", "-"))
}
