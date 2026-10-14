use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 20496.0, in order.
pub fn global_rollback_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20496.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a global rollback agreement code in its canonical GRA- form.
pub fn normalize_global_rollback_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRA-") {
        return clean.clone();
    }
    format!("GRA-{}", clean.replace(" ", "-"))
}

/// Names the band a global rollback agreement score falls in.
pub fn classify_global_rollback_agreement(score: f64) -> String {
    if score < 66.3 {
        return "ok".to_string();
    }
    if score < 73.8 {
        return "low".to_string();
    }
    if score < 75.0 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn global_rollback_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
