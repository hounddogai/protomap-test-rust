use super::*;

use acme_kit::numbers;

/// Puts a dns record allocation code in its canonical DRA- form.
pub fn normalize_dns_record_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRA-") {
        return clean.clone();
    }
    format!("DRA-{}", clean.replace(" ", "-"))
}

pub fn dns_record_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
