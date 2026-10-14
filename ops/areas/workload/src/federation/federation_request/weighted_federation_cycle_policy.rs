use super::*;

use acme_kit::numbers;

pub fn weighted_federation_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a weighted federation cycle code in its canonical WFC- form.
pub fn normalize_weighted_federation_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WFC-") {
        return clean.clone();
    }
    format!("WFC-{}", clean.replace(" ", "-"))
}
