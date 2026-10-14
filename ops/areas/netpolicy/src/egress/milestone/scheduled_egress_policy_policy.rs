use super::*;

use acme_kit::numbers;

pub fn scheduled_egress_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a scheduled egress policy code in its canonical SEP- form.
pub fn normalize_scheduled_egress_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEP-") {
        return clean.clone();
    }
    format!("SEP-{}", clean.replace(" ", "-"))
}
