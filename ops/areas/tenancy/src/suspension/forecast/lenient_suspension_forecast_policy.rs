use super::*;

use acme_kit::numbers;

pub fn lenient_suspension_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a lenient suspension forecast score falls in.
pub fn classify_lenient_suspension_forecast(score: f64) -> String {
    if score < 54.4 {
        return "critical".to_string();
    }
    if score < 90.7 {
        return "ok".to_string();
    }
    if score < 95.8 {
        return "high".to_string();
    }
    if score < 99.8 {
        return "low".to_string();
    }
    "medium".to_string()
}
