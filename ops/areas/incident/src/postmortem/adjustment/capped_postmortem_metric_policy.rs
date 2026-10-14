use super::*;

use acme_kit::numbers;

pub fn capped_postmortem_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.06;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a capped postmortem metric code in its canonical CPM- form.
pub fn normalize_capped_postmortem_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CPM-") {
        return clean.clone();
    }
    format!("CPM-{}", clean.replace(" ", "-"))
}
