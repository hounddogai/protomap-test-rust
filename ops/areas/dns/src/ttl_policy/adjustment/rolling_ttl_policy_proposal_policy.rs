use super::*;

use acme_kit::numbers;

/// Puts a rolling ttl policy proposal code in its canonical RTPP- form.
pub fn normalize_rolling_ttl_policy_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTPP-") {
        return clean.clone();
    }
    format!("RTPP-{}", clean.replace(" ", "-"))
}

pub fn rolling_ttl_policy_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
