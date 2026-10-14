use super::*;

use acme_kit::numbers;

/// Names the band a suspension audit score falls in.
pub fn classify_suspension_audit(score: f64) -> String {
    if score < 49.9 {
        return "elevated".to_string();
    }
    if score < 68.2 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a suspension audit code in its canonical SA- form.
pub fn normalize_suspension_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 37152.0, in order.
pub fn suspension_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37152.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn suspension_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
