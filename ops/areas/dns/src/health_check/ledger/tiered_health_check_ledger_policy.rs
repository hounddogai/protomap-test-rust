use super::*;

use acme_kit::numbers;

pub fn tiered_health_check_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a tiered health check ledger code in its canonical THCL- form.
pub fn normalize_tiered_health_check_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("THCL-") {
        return clean.clone();
    }
    format!("THCL-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered health check ledger score falls in.
pub fn classify_tiered_health_check_ledger(score: f64) -> String {
    if score < 19.9 {
        return "critical".to_string();
    }
    if score < 49.4 {
        return "ok".to_string();
    }
    if score < 77.4 {
        return "watch".to_string();
    }
    if score < 83.8 {
        return "high".to_string();
    }
    "minimal".to_string()
}
