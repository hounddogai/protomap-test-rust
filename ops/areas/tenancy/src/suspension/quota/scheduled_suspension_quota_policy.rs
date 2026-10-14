use super::*;

/// Names the band a scheduled suspension quota score falls in.
pub fn classify_scheduled_suspension_quota(score: f64) -> String {
    if score < 45.1 {
        return "minimal".to_string();
    }
    if score < 64.2 {
        return "elevated".to_string();
    }
    if score < 77.5 {
        return "medium".to_string();
    }
    if score < 97.8 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 77388.0, in order.
pub fn scheduled_suspension_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77388.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
