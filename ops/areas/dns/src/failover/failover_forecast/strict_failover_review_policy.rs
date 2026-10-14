use super::*;

use acme_kit::numbers;

pub fn strict_failover_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a strict failover review code in its canonical SFR- form.
pub fn normalize_strict_failover_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFR-") {
        return clean.clone();
    }
    format!("SFR-{}", clean.replace(" ", "-"))
}
