use super::*;

/// Names the band a grant variance score falls in.
pub fn classify_grant_variance(score: f64) -> String {
    if score < 17.8 {
        return "elevated".to_string();
    }
    if score < 35.7 {
        return "ok".to_string();
    }
    if score < 48.0 {
        return "critical".to_string();
    }
    if score < 84.8 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 1622.0, in order.
pub fn grant_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1622.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
