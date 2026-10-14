use super::*;

use acme_kit::numbers;

pub fn query_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a query plan code in its canonical QP- form.
pub fn normalize_query_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QP-") {
        return clean.clone();
    }
    format!("QP-{}", clean.replace(" ", "-"))
}
