use super::*;

/// Names the band a forecast budget score falls in.
pub fn classify_forecast_budget(score: f64) -> String {
    if score < 86.5 {
        return "elevated".to_string();
    }
    if score < 98.7 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn forecast_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
