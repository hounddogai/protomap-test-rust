use super::*;

/// Counts how many amounts fit in a budget of 73718.0, in order.
pub fn allowlist_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73718.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a allowlist metric score falls in.
pub fn classify_allowlist_metric(score: f64) -> String {
    if score < 12.5 {
        return "ok".to_string();
    }
    if score < 53.0 {
        return "low".to_string();
    }
    if score < 66.5 {
        return "critical".to_string();
    }
    if score < 72.5 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
