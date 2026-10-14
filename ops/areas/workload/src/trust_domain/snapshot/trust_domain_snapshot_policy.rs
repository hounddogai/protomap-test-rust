use super::*;

use acme_kit::numbers;

pub fn trust_domain_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn trust_domain_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a trust domain snapshot code in its canonical TDS- form.
pub fn normalize_trust_domain_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDS-") {
        return clean.clone();
    }
    format!("TDS-{}", clean.replace(" ", "-"))
}
