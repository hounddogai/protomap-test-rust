use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 35128.0, in order.
pub fn runbook_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35128.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn runbook_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a runbook exception score falls in.
pub fn classify_runbook_exception(score: f64) -> String {
    if score < 2.0 {
        return "elevated".to_string();
    }
    if score < 21.8 {
        return "minimal".to_string();
    }
    if score < 83.0 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Puts a runbook exception code in its canonical RE- form.
pub fn normalize_runbook_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RE-") {
        return clean.clone();
    }
    format!("RE-{}", clean.replace(" ", "-"))
}
