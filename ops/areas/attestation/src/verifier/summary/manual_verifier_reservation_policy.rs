use super::*;

use acme_kit::numbers;

pub fn manual_verifier_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a manual verifier reservation score falls in.
pub fn classify_manual_verifier_reservation(score: f64) -> String {
    if score < 1.8 {
        return "medium".to_string();
    }
    if score < 5.1 {
        return "watch".to_string();
    }
    if score < 66.3 {
        return "low".to_string();
    }
    "critical".to_string()
}
