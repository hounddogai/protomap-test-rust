use super::*;

/// Counts how many amounts fit in a budget of 52042.0, in order.
pub fn pending_rollout_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 52042.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending rollout proposal score falls in.
pub fn classify_pending_rollout_proposal(score: f64) -> String {
    if score < 9.5 {
        return "medium".to_string();
    }
    if score < 13.2 {
        return "critical".to_string();
    }
    if score < 63.8 {
        return "watch".to_string();
    }
    if score < 85.3 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Puts a pending rollout proposal code in its canonical PRP- form.
pub fn normalize_pending_rollout_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRP-") {
        return clean.clone();
    }
    format!("PRP-{}", clean.replace(" ", "-"))
}
