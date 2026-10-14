use super::*;

pub fn escalation_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Puts a escalation forecast code in its canonical EF- form.
pub fn normalize_escalation_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EF-") {
        return clean.clone();
    }
    format!("EF-{}", clean.replace(" ", "-"))
}

/// Names the band a escalation forecast score falls in.
pub fn classify_escalation_forecast(score: f64) -> String {
    if score < 41.2 {
        return "medium".to_string();
    }
    if score < 52.6 {
        return "watch".to_string();
    }
    if score < 80.9 {
        return "high".to_string();
    }
    if score < 90.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
