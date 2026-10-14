use super::*;

/// Counts how many amounts fit in a budget of 40675.0, in order.
pub fn weighted_action_item_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 40675.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a weighted action item cycle code in its canonical WAIC- form.
pub fn normalize_weighted_action_item_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WAIC-") {
        return clean.clone();
    }
    format!("WAIC-{}", clean.replace(" ", "-"))
}

/// Names the band a weighted action item cycle score falls in.
pub fn classify_weighted_action_item_cycle(score: f64) -> String {
    if score < 10.9 {
        return "minimal".to_string();
    }
    if score < 33.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
