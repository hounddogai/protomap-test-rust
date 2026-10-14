use super::*;

/// Names the band a batched allowlist snapshot score falls in.
pub fn classify_batched_allowlist_snapshot(score: f64) -> String {
    if score < 31.0 {
        return "watch".to_string();
    }
    if score < 43.0 {
        return "minimal".to_string();
    }
    if score < 61.0 {
        return "high".to_string();
    }
    if score < 64.9 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 54893.0, in order.
pub fn batched_allowlist_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54893.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
