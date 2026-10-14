use super::*;

use acme_kit::numbers;

/// Puts a strict runbook threshold code in its canonical SRT- form.
pub fn normalize_strict_runbook_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRT-") {
        return clean.clone();
    }
    format!("SRT-{}", clean.replace(" ", "-"))
}

pub fn strict_runbook_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
