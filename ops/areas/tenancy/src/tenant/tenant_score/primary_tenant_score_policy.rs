use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 37643.0, in order.
pub fn primary_tenant_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 37643.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_tenant_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a primary tenant score score falls in.
pub fn classify_primary_tenant_score(score: f64) -> String {
    if score < 33.3 {
        return "medium".to_string();
    }
    if score < 61.0 {
        return "elevated".to_string();
    }
    if score < 82.6 {
        return "critical".to_string();
    }
    if score < 99.9 {
        return "low".to_string();
    }
    "minimal".to_string()
}
