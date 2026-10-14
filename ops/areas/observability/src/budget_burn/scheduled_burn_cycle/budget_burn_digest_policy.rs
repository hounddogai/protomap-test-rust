use super::*;

use acme_kit::numbers;

/// Puts a budget burn digest code in its canonical BBD- form.
pub fn normalize_budget_burn_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBD-") {
        return clean.clone();
    }
    format!("BBD-{}", clean.replace(" ", "-"))
}

pub fn budget_burn_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.06;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
