use super::*;

use acme_kit::numbers;

pub fn scheduled_reservation_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a scheduled reservation metric code in its canonical SRM- form.
pub fn normalize_scheduled_reservation_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRM-") {
        return clean.clone();
    }
    format!("SRM-{}", clean.replace(" ", "-"))
}
