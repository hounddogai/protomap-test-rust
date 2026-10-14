use super::*;

use acme_kit::numbers;

/// Puts a rollout ledger code in its canonical RL- form.
pub fn normalize_rollout_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RL-") {
        return clean.clone();
    }
    format!("RL-{}", clean.replace(" ", "-"))
}

/// Names the band a rollout ledger score falls in.
pub fn classify_rollout_ledger(score: f64) -> String {
    if score < 6.3 {
        return "critical".to_string();
    }
    if score < 82.7 {
        return "minimal".to_string();
    }
    if score < 95.2 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn rollout_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn rollout_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
