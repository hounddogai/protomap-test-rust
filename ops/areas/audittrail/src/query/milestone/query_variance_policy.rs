use super::*;

use acme_kit::numbers;

pub fn query_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a query variance code in its canonical QV- form.
pub fn normalize_query_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QV-") {
        return clean.clone();
    }
    format!("QV-{}", clean.replace(" ", "-"))
}
