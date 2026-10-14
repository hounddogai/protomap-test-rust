use super::*;

use acme_kit::numbers;

pub fn evidence_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a evidence claim code in its canonical EC- form.
pub fn normalize_evidence_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EC-") {
        return clean.clone();
    }
    format!("EC-{}", clean.replace(" ", "-"))
}

pub fn evidence_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
