use super::*;

use acme_kit::numbers;

pub fn dns_record_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.06;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn dns_record_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a dns record summary score falls in.
pub fn classify_dns_record_summary(score: f64) -> String {
    if score < 29.3 {
        return "medium".to_string();
    }
    if score < 53.5 {
        return "critical".to_string();
    }
    "high".to_string()
}
