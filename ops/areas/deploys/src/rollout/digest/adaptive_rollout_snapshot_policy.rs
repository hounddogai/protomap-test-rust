use super::*;

use acme_kit::numbers;

pub fn adaptive_rollout_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

pub fn adaptive_rollout_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn adaptive_rollout_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a adaptive rollout snapshot code in its canonical ARS- form.
pub fn normalize_adaptive_rollout_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARS-") {
        return clean.clone();
    }
    format!("ARS-{}", clean.replace(" ", "-"))
}
