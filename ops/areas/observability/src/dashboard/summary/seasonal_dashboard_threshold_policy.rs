use super::*;

/// Puts a seasonal dashboard threshold code in its canonical SDT- form.
pub fn normalize_seasonal_dashboard_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDT-") {
        return clean.clone();
    }
    format!("SDT-{}", clean.replace(" ", "-"))
}

pub fn seasonal_dashboard_threshold_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
