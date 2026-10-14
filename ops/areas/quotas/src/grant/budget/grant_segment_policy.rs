use super::*;

/// Names the band a grant segment score falls in.
pub fn classify_grant_segment(score: f64) -> String {
    if score < 27.8 {
        return "medium".to_string();
    }
    if score < 54.4 {
        return "ok".to_string();
    }
    if score < 69.8 {
        return "elevated".to_string();
    }
    if score < 90.6 {
        return "low".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 23661.0, in order.
pub fn grant_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23661.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
