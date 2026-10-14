use super::*;

/// Names the band a audit event reservation score falls in.
pub fn classify_audit_event_reservation(score: f64) -> String {
    if score < 7.6 {
        return "high".to_string();
    }
    if score < 60.6 {
        return "medium".to_string();
    }
    if score < 72.3 {
        return "minimal".to_string();
    }
    if score < 86.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 64923.0, in order.
pub fn audit_event_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64923.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
