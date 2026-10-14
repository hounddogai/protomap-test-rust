use super::*;

use acme_kit::numbers;

/// Names the band a rolling dashboard summary score falls in.
pub fn classify_rolling_dashboard_summary(score: f64) -> String {
    if score < 41.1 {
        return "low".to_string();
    }
    if score < 48.3 {
        return "minimal".to_string();
    }
    if score < 56.1 {
        return "critical".to_string();
    }
    if score < 83.9 {
        return "watch".to_string();
    }
    "medium".to_string()
}

pub fn rolling_dashboard_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
