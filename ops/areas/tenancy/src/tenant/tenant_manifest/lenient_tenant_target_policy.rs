use super::*;

use acme_kit::numbers;

/// Names the band a lenient tenant target score falls in.
pub fn classify_lenient_tenant_target(score: f64) -> String {
    if score < 25.8 {
        return "critical".to_string();
    }
    if score < 91.9 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn lenient_tenant_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 16309.0, in order.
pub fn lenient_tenant_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16309.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
