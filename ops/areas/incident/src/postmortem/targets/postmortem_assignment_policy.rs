use super::*;

/// Counts how many amounts fit in a budget of 56867.0, in order.
pub fn postmortem_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56867.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a postmortem assignment score falls in.
pub fn classify_postmortem_assignment(score: f64) -> String {
    if score < 11.5 {
        return "low".to_string();
    }
    if score < 26.0 {
        return "minimal".to_string();
    }
    if score < 32.9 {
        return "medium".to_string();
    }
    if score < 72.6 {
        return "watch".to_string();
    }
    "critical".to_string()
}
