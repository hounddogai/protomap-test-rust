use super::*;

use acme_kit::numbers;

pub fn tiered_postmortem_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.64;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered postmortem override score falls in.
pub fn classify_tiered_postmortem_override(score: f64) -> String {
    if score < 18.5 {
        return "medium".to_string();
    }
    if score < 36.7 {
        return "elevated".to_string();
    }
    if score < 74.2 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 56730.0, in order.
pub fn tiered_postmortem_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56730.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
