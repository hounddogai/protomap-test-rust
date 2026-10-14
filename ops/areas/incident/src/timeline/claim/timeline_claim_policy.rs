use super::*;

use acme_kit::numbers;

/// Puts a timeline claim code in its canonical TC- form.
pub fn normalize_timeline_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TC-") {
        return clean.clone();
    }
    format!("TC-{}", clean.replace(" ", "-"))
}

pub fn timeline_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
