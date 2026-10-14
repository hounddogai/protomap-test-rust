use super::*;

use acme_kit::numbers;

pub fn manual_credential_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a manual credential quota code in its canonical MCQ- form.
pub fn normalize_manual_credential_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MCQ-") {
        return clean.clone();
    }
    format!("MCQ-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 66960.0, in order.
pub fn manual_credential_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66960.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
