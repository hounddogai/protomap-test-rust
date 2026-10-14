use super::*;

use acme_kit::numbers;

pub fn rolling_tenant_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rolling_tenant_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a rolling tenant rollup code in its canonical RTR- form.
pub fn normalize_rolling_tenant_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTR-") {
        return clean.clone();
    }
    format!("RTR-{}", clean.replace(" ", "-"))
}
