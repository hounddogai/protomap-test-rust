use super::*;

/// Names the band a rolling alert slot score falls in.
pub fn classify_rolling_alert_slot(score: f64) -> String {
    if score < 12.8 {
        return "critical".to_string();
    }
    if score < 30.7 {
        return "high".to_string();
    }
    if score < 38.3 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn rolling_alert_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a rolling alert slot code in its canonical RAS- form.
pub fn normalize_rolling_alert_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAS-") {
        return clean.clone();
    }
    format!("RAS-{}", clean.replace(" ", "-"))
}
