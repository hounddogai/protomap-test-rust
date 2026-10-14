use super::*;

/// Counts how many amounts fit in a budget of 51045.0, in order.
pub fn audit_event_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51045.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a audit event score score falls in.
pub fn classify_audit_event_score(score: f64) -> String {
    if score < 27.8 {
        return "medium".to_string();
    }
    if score < 71.0 {
        return "ok".to_string();
    }
    if score < 79.9 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
