use super::*;

use acme_kit::numbers;

pub fn scheduled_grant_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn scheduled_grant_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
