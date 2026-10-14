use super::*;

use acme_kit::numbers;

pub fn rule_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a rule claim code in its canonical RC- form.
pub fn normalize_rule_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}
