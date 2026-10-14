use super::*;

use acme_kit::numbers;

/// Puts a lenient tenant quota code in its canonical LTQ- form.
pub fn normalize_lenient_tenant_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LTQ-") {
        return clean.clone();
    }
    format!("LTQ-{}", clean.replace(" ", "-"))
}

pub fn lenient_tenant_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn lenient_tenant_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
