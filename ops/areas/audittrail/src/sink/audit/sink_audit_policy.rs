use super::*;

use acme_kit::numbers;

/// Puts a sink audit code in its canonical SA- form.
pub fn normalize_sink_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}

pub fn sink_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
