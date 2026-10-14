use super::*;

use acme_kit::numbers;

/// Puts a fallback postmortem entry code in its canonical FPE- form.
pub fn normalize_fallback_postmortem_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPE-") {
        return clean.clone();
    }
    format!("FPE-{}", clean.replace(" ", "-"))
}

pub fn fallback_postmortem_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_postmortem_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
