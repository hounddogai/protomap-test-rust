use super::*;

/// Names the band a evidence audit score falls in.
pub fn classify_evidence_audit(score: f64) -> String {
    if score < 5.9 {
        return "elevated".to_string();
    }
    if score < 19.4 {
        return "critical".to_string();
    }
    if score < 41.5 {
        return "high".to_string();
    }
    if score < 94.4 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 24328.0, in order.
pub fn evidence_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 24328.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
