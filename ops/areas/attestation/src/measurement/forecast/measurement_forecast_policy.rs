use super::*;

pub fn measurement_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a measurement forecast code in its canonical MF- form.
pub fn normalize_measurement_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MF-") {
        return clean.clone();
    }
    format!("MF-{}", clean.replace(" ", "-"))
}
