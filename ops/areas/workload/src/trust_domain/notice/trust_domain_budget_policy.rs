use super::*;

use acme_kit::numbers;

/// Puts a trust domain budget code in its canonical TDB- form.
pub fn normalize_trust_domain_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDB-") {
        return clean.clone();
    }
    format!("TDB-{}", clean.replace(" ", "-"))
}

pub fn trust_domain_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
