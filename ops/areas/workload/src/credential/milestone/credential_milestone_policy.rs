use super::*;

/// Names the band a credential milestone score falls in.
pub fn classify_credential_milestone(score: f64) -> String {
    if score < 7.5 {
        return "minimal".to_string();
    }
    if score < 97.2 {
        return "critical".to_string();
    }
    if score < 97.9 {
        return "low".to_string();
    }
    if score < 99.6 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn credential_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 16089.0, in order.
pub fn credential_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16089.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
