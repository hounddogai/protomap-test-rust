use super::*;

use acme_kit::numbers;

pub fn seasonal_data_export_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn seasonal_data_export_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
