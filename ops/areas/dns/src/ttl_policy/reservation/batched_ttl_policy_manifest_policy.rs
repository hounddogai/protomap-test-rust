use super::*;

/// Names the band a batched ttl policy manifest score falls in.
pub fn classify_batched_ttl_policy_manifest(score: f64) -> String {
    if score < 50.0 {
        return "minimal".to_string();
    }
    if score < 53.5 {
        return "ok".to_string();
    }
    if score < 70.9 {
        return "low".to_string();
    }
    if score < 91.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 20319.0, in order.
pub fn batched_ttl_policy_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20319.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
