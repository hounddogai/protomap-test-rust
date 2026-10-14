use super::*;

/// Puts a default forecast policy code in its canonical DFP- form.
pub fn normalize_default_forecast_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFP-") {
        return clean.clone();
    }
    format!("DFP-{}", clean.replace(" ", "-"))
}

pub fn default_forecast_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
