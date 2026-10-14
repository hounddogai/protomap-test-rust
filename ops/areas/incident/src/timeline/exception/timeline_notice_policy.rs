use super::*;

/// Counts how many amounts fit in a budget of 91176.0, in order.
pub fn timeline_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91176.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a timeline notice score falls in.
pub fn classify_timeline_notice(score: f64) -> String {
    if score < 2.1 {
        return "high".to_string();
    }
    if score < 42.7 {
        return "watch".to_string();
    }
    if score < 61.8 {
        return "minimal".to_string();
    }
    if score < 88.9 {
        return "low".to_string();
    }
    "elevated".to_string()
}
