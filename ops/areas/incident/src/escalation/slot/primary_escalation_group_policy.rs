use super::*;

use acme_kit::numbers;

/// Puts a primary escalation group code in its canonical PEG- form.
pub fn normalize_primary_escalation_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PEG-") {
        return clean.clone();
    }
    format!("PEG-{}", clean.replace(" ", "-"))
}

pub fn primary_escalation_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
