use super::*;

use acme_kit::numbers;

/// Puts a tenant schedule code in its canonical TS- form.
pub fn normalize_tenant_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TS-") {
        return clean.clone();
    }
    format!("TS-{}", clean.replace(" ", "-"))
}

pub fn tenant_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn tenant_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a tenant schedule score falls in.
pub fn classify_tenant_schedule(score: f64) -> String {
    if score < 55.0 {
        return "high".to_string();
    }
    if score < 57.9 {
        return "ok".to_string();
    }
    if score < 86.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
