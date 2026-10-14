use super::*;

use acme_kit::numbers;

/// Names the band a fallback tenant rule score falls in.
pub fn classify_fallback_tenant_rule(score: f64) -> String {
    if score < 3.7 {
        return "elevated".to_string();
    }
    if score < 65.9 {
        return "critical".to_string();
    }
    if score < 79.8 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 56565.0, in order.
pub fn fallback_tenant_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56565.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_tenant_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
