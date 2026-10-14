use super::*;

use acme_kit::numbers;

pub fn reservation_window_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a reservation window score falls in.
pub fn classify_reservation_window(score: f64) -> String {
    if score < 26.4 {
        return "ok".to_string();
    }
    if score < 88.5 {
        return "medium".to_string();
    }
    "watch".to_string()
}
