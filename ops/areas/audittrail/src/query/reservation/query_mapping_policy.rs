use super::*;

use acme_kit::numbers;

/// Puts a query mapping code in its canonical QM- form.
pub fn normalize_query_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QM-") {
        return clean.clone();
    }
    format!("QM-{}", clean.replace(" ", "-"))
}

pub fn query_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn query_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
