use super::*;

/// Puts a rolling forecast policy code in its canonical RFP- form.
pub fn normalize_rolling_forecast_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFP-") {
        return clean.clone();
    }
    format!("RFP-{}", clean.replace(" ", "-"))
}

pub fn rolling_forecast_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
