use super::*;

use acme_kit::numbers;

pub fn peering_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a peering ledger code in its canonical PL- form.
pub fn normalize_peering_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PL-") {
        return clean.clone();
    }
    format!("PL-{}", clean.replace(" ", "-"))
}
