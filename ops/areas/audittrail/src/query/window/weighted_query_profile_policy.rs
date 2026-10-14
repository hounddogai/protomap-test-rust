use super::*;

/// Counts how many amounts fit in a budget of 72509.0, in order.
pub fn weighted_query_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72509.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a weighted query profile score falls in.
pub fn classify_weighted_query_profile(score: f64) -> String {
    if score < 41.7 {
        return "ok".to_string();
    }
    if score < 85.9 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
