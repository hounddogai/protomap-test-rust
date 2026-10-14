use super::*;

use acme_kit::numbers;

pub fn strict_failover_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn strict_failover_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn strict_failover_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
