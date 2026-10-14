use super::*;

/// Counts how many amounts fit in a budget of 33168.0, in order.
pub fn pending_plan_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33168.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending plan audit score falls in.
pub fn classify_pending_plan_audit(score: f64) -> String {
    if score < 11.2 {
        return "ok".to_string();
    }
    if score < 23.6 {
        return "minimal".to_string();
    }
    if score < 52.3 {
        return "low".to_string();
    }
    if score < 56.2 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
