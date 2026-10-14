use super::*;

use acme_kit::numbers;

/// Puts a default alert slot code in its canonical DAS- form.
pub fn normalize_default_alert_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DAS-") {
        return clean.clone();
    }
    format!("DAS-{}", clean.replace(" ", "-"))
}

pub fn default_alert_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
