use super::*;

use acme_kit::numbers;

/// Puts a tenant window code in its canonical TW- form.
pub fn normalize_tenant_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TW-") {
        return clean.clone();
    }
    format!("TW-{}", clean.replace(" ", "-"))
}

pub fn tenant_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
