use super::*;

use acme_kit::numbers;

/// Puts a batched forecast cycle code in its canonical BFC- form.
pub fn normalize_batched_forecast_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BFC-") {
        return clean.clone();
    }
    format!("BFC-{}", clean.replace(" ", "-"))
}

/// Names the band a batched forecast cycle score falls in.
pub fn classify_batched_forecast_cycle(score: f64) -> String {
    if score < 8.2 {
        return "ok".to_string();
    }
    if score < 27.2 {
        return "critical".to_string();
    }
    if score < 49.6 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn batched_forecast_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn batched_forecast_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
