use super::*;

use acme_kit::numbers;

/// Puts a fallback failover budget code in its canonical FFB- form.
pub fn normalize_fallback_failover_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FFB-") {
        return clean.clone();
    }
    format!("FFB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 19203.0, in order.
pub fn fallback_failover_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19203.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_failover_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback failover budget score falls in.
pub fn classify_fallback_failover_budget(score: f64) -> String {
    if score < 6.5 {
        return "low".to_string();
    }
    if score < 92.0 {
        return "watch".to_string();
    }
    if score < 95.2 {
        return "ok".to_string();
    }
    "high".to_string()
}
