use super::*;

use acme_kit::numbers;

pub fn tiered_egress_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a tiered egress milestone code in its canonical TEM- form.
pub fn normalize_tiered_egress_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TEM-") {
        return clean.clone();
    }
    format!("TEM-{}", clean.replace(" ", "-"))
}

pub fn tiered_egress_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
