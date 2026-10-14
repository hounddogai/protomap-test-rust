use super::*;

use acme_kit::numbers;

/// Puts a tenant proposal code in its canonical TP- form.
pub fn normalize_tenant_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TP-") {
        return clean.clone();
    }
    format!("TP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 39434.0, in order.
pub fn tenant_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39434.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tenant_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tenant proposal score falls in.
pub fn classify_tenant_proposal(score: f64) -> String {
    if score < 29.9 {
        return "minimal".to_string();
    }
    if score < 68.8 {
        return "watch".to_string();
    }
    if score < 74.9 {
        return "ok".to_string();
    }
    if score < 89.3 {
        return "high".to_string();
    }
    "medium".to_string()
}
