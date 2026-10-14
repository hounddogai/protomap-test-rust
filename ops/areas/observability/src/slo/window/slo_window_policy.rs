use super::*;

use acme_kit::numbers;

/// Names the band a slo window score falls in.
pub fn classify_slo_window(score: f64) -> String {
    if score < 9.8 {
        return "ok".to_string();
    }
    if score < 21.3 {
        return "minimal".to_string();
    }
    if score < 42.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn slo_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
