use super::*;

use acme_kit::numbers;

/// Puts a weighted token exception code in its canonical WTE- form.
pub fn normalize_weighted_token_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WTE-") {
        return clean.clone();
    }
    format!("WTE-{}", clean.replace(" ", "-"))
}

pub fn weighted_token_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
