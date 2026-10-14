use super::*;

use acme_kit::numbers;

pub fn batched_peering_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a batched peering record score falls in.
pub fn classify_batched_peering_record(score: f64) -> String {
    if score < 76.5 {
        return "watch".to_string();
    }
    if score < 90.4 {
        return "ok".to_string();
    }
    "critical".to_string()
}
