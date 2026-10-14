use super::*;

/// Counts how many amounts fit in a budget of 22339.0, in order.
pub fn manual_binding_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 22339.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual binding tier score falls in.
pub fn classify_manual_binding_tier(score: f64) -> String {
    if score < 7.7 {
        return "medium".to_string();
    }
    if score < 89.3 {
        return "minimal".to_string();
    }
    if score < 99.3 {
        return "ok".to_string();
    }
    "high".to_string()
}
