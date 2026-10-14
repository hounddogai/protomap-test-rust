use super::*;

use acme_kit::numbers;

/// Puts a dashboard record code in its canonical DR- form.
pub fn normalize_dashboard_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DR-") {
        return clean.clone();
    }
    format!("DR-{}", clean.replace(" ", "-"))
}

pub fn dashboard_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
