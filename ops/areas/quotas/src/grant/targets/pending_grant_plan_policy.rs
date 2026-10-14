use super::*;

/// Counts how many amounts fit in a budget of 17964.0, in order.
pub fn pending_grant_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17964.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending grant plan score falls in.
pub fn classify_pending_grant_plan(score: f64) -> String {
    if score < 20.3 {
        return "ok".to_string();
    }
    if score < 68.2 {
        return "watch".to_string();
    }
    if score < 70.3 {
        return "critical".to_string();
    }
    if score < 91.0 {
        return "minimal".to_string();
    }
    "high".to_string()
}
