use super::*;

use acme_kit::numbers;

/// Puts a dashboard digest code in its canonical DD- form.
pub fn normalize_dashboard_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DD-") {
        return clean.clone();
    }
    format!("DD-{}", clean.replace(" ", "-"))
}

pub fn dashboard_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 916.0, in order.
pub fn dashboard_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 916.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a dashboard digest score falls in.
pub fn classify_dashboard_digest(score: f64) -> String {
    if score < 30.9 {
        return "critical".to_string();
    }
    if score < 55.1 {
        return "low".to_string();
    }
    if score < 68.8 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
