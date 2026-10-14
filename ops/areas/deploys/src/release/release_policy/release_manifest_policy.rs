use super::*;

/// Counts how many amounts fit in a budget of 30917.0, in order.
pub fn release_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30917.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a release manifest score falls in.
pub fn classify_release_manifest(score: f64) -> String {
    if score < 25.4 {
        return "elevated".to_string();
    }
    if score < 28.8 {
        return "low".to_string();
    }
    if score < 78.8 {
        return "high".to_string();
    }
    if score < 87.7 {
        return "critical".to_string();
    }
    "ok".to_string()
}
