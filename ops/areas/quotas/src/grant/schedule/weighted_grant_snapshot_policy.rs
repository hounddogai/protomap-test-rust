use super::*;

use acme_kit::numbers;

pub fn weighted_grant_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a weighted grant snapshot code in its canonical WGS- form.
pub fn normalize_weighted_grant_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WGS-") {
        return clean.clone();
    }
    format!("WGS-{}", clean.replace(" ", "-"))
}
