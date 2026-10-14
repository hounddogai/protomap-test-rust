use super::*;

use acme_kit::numbers;

/// Puts a rolling rollout snapshot code in its canonical RRS- form.
pub fn normalize_rolling_rollout_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRS-") {
        return clean.clone();
    }
    format!("RRS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 28519.0, in order.
pub fn rolling_rollout_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28519.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rolling_rollout_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling rollout snapshot score falls in.
pub fn classify_rolling_rollout_snapshot(score: f64) -> String {
    if score < 4.8 {
        return "medium".to_string();
    }
    if score < 81.0 {
        return "critical".to_string();
    }
    if score < 89.4 {
        return "watch".to_string();
    }
    "high".to_string()
}
