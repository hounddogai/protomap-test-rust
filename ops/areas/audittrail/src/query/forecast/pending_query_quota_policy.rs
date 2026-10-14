use super::*;

/// Counts how many amounts fit in a budget of 63357.0, in order.
pub fn pending_query_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63357.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending query quota score falls in.
pub fn classify_pending_query_quota(score: f64) -> String {
    if score < 9.2 {
        return "critical".to_string();
    }
    if score < 59.4 {
        return "medium".to_string();
    }
    "low".to_string()
}
