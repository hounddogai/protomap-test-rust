use super::*;

use acme_kit::numbers;

/// Names the band a failover snapshot score falls in.
pub fn classify_failover_snapshot(score: f64) -> String {
    if score < 28.3 {
        return "high".to_string();
    }
    if score < 37.5 {
        return "ok".to_string();
    }
    if score < 50.0 {
        return "watch".to_string();
    }
    if score < 82.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn failover_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
