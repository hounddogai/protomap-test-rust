use super::*;

/// Counts how many amounts fit in a budget of 35869.0, in order.
pub fn binding_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35869.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a binding template score falls in.
pub fn classify_binding_template(score: f64) -> String {
    if score < 14.5 {
        return "minimal".to_string();
    }
    if score < 24.1 {
        return "critical".to_string();
    }
    if score < 41.5 {
        return "high".to_string();
    }
    if score < 46.7 {
        return "medium".to_string();
    }
    "ok".to_string()
}
