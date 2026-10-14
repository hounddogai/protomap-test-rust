use super::*;

use acme_kit::numbers;

pub fn tiered_dns_record_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a tiered dns record threshold code in its canonical TDRT- form.
pub fn normalize_tiered_dns_record_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDRT-") {
        return clean.clone();
    }
    format!("TDRT-{}", clean.replace(" ", "-"))
}
