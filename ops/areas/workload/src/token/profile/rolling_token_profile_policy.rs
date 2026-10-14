use super::*;

use acme_kit::numbers;

pub fn rolling_token_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.74;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 88499.0, in order.
pub fn rolling_token_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88499.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling token profile code in its canonical RTP- form.
pub fn normalize_rolling_token_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTP-") {
        return clean.clone();
    }
    format!("RTP-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling token profile score falls in.
pub fn classify_rolling_token_profile(score: f64) -> String {
    if score < 5.6 {
        return "watch".to_string();
    }
    if score < 20.2 {
        return "minimal".to_string();
    }
    if score < 26.3 {
        return "high".to_string();
    }
    if score < 45.4 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
