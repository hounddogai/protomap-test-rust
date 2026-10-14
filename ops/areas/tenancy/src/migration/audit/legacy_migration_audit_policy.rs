use super::*;

use acme_kit::numbers;

/// Names the band a legacy migration audit score falls in.
pub fn classify_legacy_migration_audit(score: f64) -> String {
    if score < 11.8 {
        return "medium".to_string();
    }
    if score < 99.8 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn legacy_migration_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a legacy migration audit code in its canonical LMA- form.
pub fn normalize_legacy_migration_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LMA-") {
        return clean.clone();
    }
    format!("LMA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 53103.0, in order.
pub fn legacy_migration_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53103.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
