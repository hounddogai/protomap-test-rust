use super::*;

use acme_kit::numbers;

/// Names the band a seasonal dns record reservation score falls in.
pub fn classify_seasonal_dns_record_reservation(score: f64) -> String {
    if score < 13.1 {
        return "medium".to_string();
    }
    if score < 43.6 {
        return "critical".to_string();
    }
    if score < 51.2 {
        return "low".to_string();
    }
    if score < 95.1 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn seasonal_dns_record_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
