use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 83514.0, in order.
pub fn lenient_allowlist_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83514.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_allowlist_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient allowlist estimate score falls in.
pub fn classify_lenient_allowlist_estimate(score: f64) -> String {
    if score < 23.2 {
        return "high".to_string();
    }
    if score < 25.3 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Puts a lenient allowlist estimate code in its canonical LAE- form.
pub fn normalize_lenient_allowlist_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAE-") {
        return clean.clone();
    }
    format!("LAE-{}", clean.replace(" ", "-"))
}
