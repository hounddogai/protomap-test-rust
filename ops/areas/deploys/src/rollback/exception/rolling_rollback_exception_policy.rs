use super::*;

use acme_kit::numbers;

pub fn rolling_rollback_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a rolling rollback exception code in its canonical RRE- form.
pub fn normalize_rolling_rollback_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRE-") {
        return clean.clone();
    }
    format!("RRE-{}", clean.replace(" ", "-"))
}
