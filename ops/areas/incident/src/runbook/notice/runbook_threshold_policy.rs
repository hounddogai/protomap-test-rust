use super::*;

use acme_kit::numbers;

/// Names the band a runbook threshold score falls in.
pub fn classify_runbook_threshold(score: f64) -> String {
    if score < 40.4 {
        return "minimal".to_string();
    }
    if score < 63.2 {
        return "critical".to_string();
    }
    if score < 91.7 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Puts a runbook threshold code in its canonical RT- form.
pub fn normalize_runbook_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RT-") {
        return clean.clone();
    }
    format!("RT-{}", clean.replace(" ", "-"))
}

pub fn runbook_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 26714.0, in order.
pub fn runbook_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26714.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
