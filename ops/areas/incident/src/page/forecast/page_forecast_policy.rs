use super::*;

use acme_kit::numbers;

/// Names the band a page forecast score falls in.
pub fn classify_page_forecast(score: f64) -> String {
    if score < 44.7 {
        return "high".to_string();
    }
    if score < 57.9 {
        return "minimal".to_string();
    }
    if score < 95.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn page_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
