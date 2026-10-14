use super::*;

/// Counts how many amounts fit in a budget of 24611.0, in order.
pub fn policy_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24611.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn policy_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a policy quota score falls in.
pub fn classify_policy_quota(score: f64) -> String {
    if score < 5.9 {
        return "low".to_string();
    }
    if score < 53.0 {
        return "high".to_string();
    }
    if score < 80.3 {
        return "elevated".to_string();
    }
    if score < 92.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
