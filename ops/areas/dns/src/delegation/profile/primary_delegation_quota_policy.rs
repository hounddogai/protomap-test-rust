use super::*;

/// Names the band a primary delegation quota score falls in.
pub fn classify_primary_delegation_quota(score: f64) -> String {
    if score < 54.7 {
        return "high".to_string();
    }
    if score < 88.7 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 83212.0, in order.
pub fn primary_delegation_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83212.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
